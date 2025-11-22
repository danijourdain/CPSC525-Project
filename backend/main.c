
#include <stdlib.h>
#include <stdio.h>
#include "structs.h"
#include <errno.h>
#include <linux/futex.h> /* Definition of FUTEX_* constants */
#include <sys/syscall.h> /* Definition of SYS_* constants */
#include <unistd.h>
#include <pthread.h>
#include <string.h>
#include <stdatomic.h>
#include <limits.h>
#include "signals/signal.h"
#include "main.h"
#include "master/master.h"
#include <openssl/evp.h>
#include <sched.h>
// BUFFER




/// @brief Opens a new order book server with a specific ID.
/// @param id the ID of the order server open.
/// @return The pointer to the server handle.
SubjugateOrderBook *open_server(int id, MasterBook *master)
{
    SubjugateOrderBook *ptr = (SubjugateOrderBook *)malloc(sizeof(SubjugateOrderBook));

    if(get_region_name(id) == NULL) {
        // This is an invalid entry.
        return NULL;
    }

    // Set base fields to identify the order book + the current order status.
    ptr->id = id;
    ptr->current_order.status = 0;
    ptr->master = master;

    pthread_mutex_init(&ptr->fixer_tex, NULL);


    // We set the security level to low for now.
    ptr->security_level = SEC_MID;
    init_buffer(&ptr->current);




    return ptr;
}



/// @brief How many iterations of hashing should be used to check the passwords.
/// @param level The 
/// @return 
int iterations_for_level(SecLevel level) {
    if(level == SEC_LOW) {
        return 1;
    } else if(level == SEC_MID) {
        return 1000;
    } else if(level == SEC_HIGH) {
        return 100000;
    } else if(level == SEC_VERYHIGH) {
        return 100000;
    }
}



/// @brief Checks if the server is locked.
/// @param handle the handle to the orderbook server.
/// @return if the handle is locked or not.
int check_locked(SubjugateOrderBook *handle)
{
    return handle->ctrl != 0;
}





/// @brief This was taken from A3
/// @param str 
/// @return 
static const char * hashString(const char * str, char *resbuf)
{
    EVP_MD_CTX * context = EVP_MD_CTX_create();
    if (! context) error(-1, 0, "failed EVP_MD_CTX_create");
    if (! EVP_DigestInit_ex(context, EVP_sha256(), NULL))
        error(-1, 0, "failed EVP_DigestInit_ex");
    if (! EVP_DigestUpdate(context, str, strlen(str)))
        error(-1, 0, "failed EVP_DigestUpdate");
    unsigned char hashBuff[EVP_MAX_MD_SIZE];
    unsigned int hashLen = 0;
    if (! EVP_DigestFinal_ex(context, hashBuff, &hashLen))
        error(-1, 0, "failed EVP_DigestFinal_ex");
    for (unsigned int i = 0; i < hashLen; ++i) {
        sprintf(resbuf + i * 2, "%02x", hashBuff[i]);
    }
    EVP_MD_CTX_destroy(context);
    return resbuf;
}


/// @brief Performs N hashes.
/// @param str The source to hash.
/// @param target The target to hash into.
/// @param iterations How many iterations we should run.
void hashIteratively(char *str, char target[EVP_MAX_MD_SIZE * 2], int iterations) {

    char source[EVP_MAX_MD_SIZE * 2];
    char dest[EVP_MAX_MD_SIZE * 2];

    // Copy into the source buffer.
    strncpy(source, str, sizeof(source));

    for(int i = 0; i < iterations; i++) {
        
        
        // Perform a hash.
        hashString(source, dest);


        // Transfer the destination register to the source register.
        strncpy(source, dest, sizeof(dest));
        
        // Yield to the scheduler.
        sched_yield();
    }


    // Copy the final result to the target buffer.
    strncpy(target, dest, sizeof(dest));
}

/// @brief Checks if the password is in alignment with the region password.
/// @param level The level to check the password at.
/// @param region The region to check it for.
/// @param password The password to check.
/// @return A boolean value determining if the password was valid or not.
int check_region_password(
    SecLevel level,
    int region,
    char *password 
) {

    // Start by getting the region password.
    char *region_pwd = get_region_password(region);
    if(region_pwd == NULL) {
        // We cannot possibly compare with a null string.
        return 0;
    }

    // Do the base hash. Everything is relative to this.
    char basepwd[EVP_MAX_MD_SIZE * 2];
    hashString(password, basepwd);

    if(level == SEC_NONE) {
        // John: For debug setups where we want to test the order book functionality
        // without dealing w/ auth latency.
        return 1;
    } else if(level == SEC_LOW) {
        // In this case we just need to do the comparison;
        return strncmp(basepwd, region_pwd, sizeof(basepwd)) == 0;
    } else {
        // Determine how many iterations we need.
        int iterations = iterations_for_level(level);

        // Expand the region password.
        char region_pwd_expanded[EVP_MAX_MD_SIZE * 2];
        hashIteratively(region_pwd, region_pwd_expanded, iterations);


        // Expand the user's provided password.
        char user_pwd_expanded[EVP_MAX_MD_SIZE * 2];
        hashIteratively(basepwd, user_pwd_expanded, iterations);

        // Now we do the comparison.
        return strncmp(user_pwd_expanded, region_pwd_expanded, sizeof(user_pwd_expanded)) == 0;

    }

    return 0;
}



int try_lock(SubjugateOrderBook *handle, char *password)
{
    
    // Increase the request count.
    handle->req_count += 1;

    // Check if we are already LOCKED.
    if (handle->ctrl == 2)
    {
        // We are locked, so we error and return a value.
        // TODO: Switch return types to the correct ones.
        errno = EBUSY;
        return 0;
    }

    // We indicate that we are currently doing stuff with the dataabse.
    handle->ctrl = 1;


    // Check if we have high traffic.
    int high_traffic_mode = handle->req_count > 25;

    // If we have high traffic let's be a bit more
    // liberal with the hashing as this is already enough.
    if(high_traffic_mode) {
        handle->security_level = (handle->security_level >> 1) & !3;
    }

    int result = check_region_password(handle->security_level, handle->id, password);


    // Make sure we bump this back down if we are in high traffic mnode.
    if(high_traffic_mode) {
        handle->security_level = (handle->security_level << 1) & !3;
    }

    // Now we check the password and if it verified correctly.
    if(!result) {
        errno = EACCES;
        handle->ctrl = 0;
        return 0;
    }

    // We are done and grant access
    handle->ctrl = 2;


    return 1;
}

void release_lock(SubjugateOrderBook *handle, __uint32_t claimant)
{
    
        handle->ctrl = 0;
        handle->user_id = 0;
    
}

/// @brief Fetches the current user using the database.
/// @param handle The pointer to the order book.
/// @return The user_id of the user using the application.
__uint32_t fetch_current_user(SubjugateOrderBook *handle)
{
    return handle->user_id;
}

int open_record(SubjugateOrderBook *handle)
{

    if (handle->current_order.status != 0)
    {
        return -1; // We are currently processing an order.
    }
    
    // The ID of the region.
    handle->current_order.region = handle->id;

    // Set the current record to open.
    handle->current_order.status = 1;

    return 1;
}



int set_recipient(SubjugateOrderBook *handle, int id)
{
    if (handle->current_order.status != 1)
    {
        return -1; // We are not currently processing an order.
    }

    // Set the current record's recipient.
    handle->current_order.recipient = id;

    return 1;
}

int set_sender(SubjugateOrderBook *handle, int id)
{
    if (handle->current_order.status != 1)
    {
        return -1; // We are not currently processing an order.
    }

    // Set the current record's sender.
    handle->current_order.sender = id;

    return 1;
}

int set_money(SubjugateOrderBook *handle, int id)
{
    if (handle->current_order.status != 1)
    {
        return -1; // We are not currently processing an order.
    }

    // Set the current record's sender.
    handle->current_order.money = id;

    return 1;
}





void print_order(Order ptr)
{
    printf("Order { sender: %d, recipient: %d, money: %d }", ptr.sender, ptr.recipient, ptr.money);
}

void print_buffer(Buffer *src)
{
    printf("Buffer { pos: %d, buffer: [", src->pos);

    for (int i = 0; i < src->pos; i++)
    {
        print_order(src->orders[i]);
        if (i != src->pos - 1)
        {
            printf(", ");
        }
    }

    printf("]}\n");
}

int flush_order(SubjugateOrderBook *handle)
{
    if (buffer_full(&handle->current))
    {
        return -1; // no room to flush orders.
    }
    if (handle->current_order.status != 1)
    {
        return -1; // current order is not closed out.
    }

    // Write it to the local buffer.
    buffer_push(&handle->current, handle->current_order);
    handle->current_order.status = 0;


    // Push the records to the master record.
    // John: Maybe in the future we can add a bit
    // more balancing here so it waits until the buffer
    // is more full before pushing?
    push_records(handle->master, &handle->current);
    return 0;
}

void log_last_order(SubjugateOrderBook *handle)
{
    if (buffer_pos(&handle->current) == 0)
    {
        printf("log: nothing to show.\n");
    }
    else
    {
        Order entry = handle->current.orders[handle->current.pos - 1];
        printf("log: %d -> %d ($%d)\n", entry.sender, entry.recipient, entry.money);
    }
}

/// @brief Closes the server and frees the resources.
/// @param handle the handle to the server.
/// @return if it is succesful
int close_server(SubjugateOrderBook *handle)
{

    // For the good code.
    pthread_mutex_destroy(&handle->fixer_tex);

    // Free the allocation by the server.
    free((void *)handle);
    return 1;
}