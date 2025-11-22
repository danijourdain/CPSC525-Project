
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


    // We set the security level to low for now.
    ptr->security_level = SEC_LOW;
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
        return 10;
    } else if(level == SEC_HIGH) {
        return 10000;
    } else if(level == SEC_VERYHIGH) {
        return 1000000;
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
    // static char resbuf[EVP_MAX_MD_SIZE * 2];
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

    if(level == SEC_LOW) {
        // In this case we just need to do the comparison;
        return strncmp(basepwd, region_pwd, sizeof(basepwd)) == 0;
    } else {
        int iterations = iterations_for_level(level);

        printf("will apply %d iterations\n", iterations);


        // Expand the region password.
        char region_pwd_expanded[EVP_MAX_MD_SIZE * 2];
        hashIteratively(region_pwd, region_pwd_expanded, iterations);


        // Expand the user's provided password.
        



    }

    return 0;
}



int try_lock(SubjugateOrderBook *handle, char *password)
{

    printf("Attempting to lock with password %s\n", password);
    if (handle->ctrl == 2)
    {
        errno = EBUSY;
        return 0;
    }

    // We indicate that we are currently doing stuff with the dataabse.
    handle->ctrl = 1;

    //  const char *msg = "hello world";


    //  char *reso = hashString(msg);
    //  printf("reso: %s\n", reso);

    handle->security_level = SEC_HIGH;
    int result = check_region_password(handle->security_level, handle->id, password);
    printf("result: %d\n", result);
    //  unsigned char digest[SHA256_DIGEST_LENGTH];

    

    // // Compute SHA-256
    // SHA256((const unsigned char *)msg, strlen(msg), digest);

    // // Print as hex
    // for (int i = 0; i < SHA256_DIGEST_LENGTH; i++) {
    //     printf("%02x", digest[i]);
    // }
    // printf("\n");

    for (int i = 0; i < 10000; i++)
    {
        // doing some sort of lookup lmao.
    }

    // We are done and grant access
    // THIS INTRODUCES A VULNERABILITY
    handle->ctrl = 2;

    handle->user_id = handle->id;

    return 1;
}

void release_lock(SubjugateOrderBook *handle, __uint32_t claimant)
{
    if (handle->user_id == claimant)
    {
        handle->ctrl = 0;
        handle->user_id = 0;
    }
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
    buffer_push(&handle->current, handle->current_order);
    handle->current_order.status = 0;


    push_records(handle->master, &handle->current);
    
    // printf("op: %d\n", handle->current.pos);

    // print_buffer(&handle->current);


    // wait_signal(&handle->signal, 0);
    // transfer_buffers(&handle->current, &handle->background);
    // set_signal(&handle->signal, 1);

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

    // Commit to the background thread.
    // commit_to_background_thread(handle);

    // Free the allocation by the server.
    free((void *)handle);
    return 1;
}