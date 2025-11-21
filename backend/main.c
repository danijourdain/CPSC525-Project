
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
// BUFFER



void background_thread(ServerT *handle)
{
    printf("background thread started\n");

    while (1)
    {
        wait_signal(&handle->signal, 1);
        printf("Received a packet.\n");

        set_signal(&handle->signal, 0);
    }
}

/// @brief Opens a new order book server with a specific ID.
/// @param id the ID of the order server open.
/// @return The pointer to the server handle.
ServerT *open_server(int id)
{
    ServerT *ptr = (ServerT *)malloc(sizeof(ServerT));
    ptr->id = id;
    ptr->current_order.status = 0;
    init_buffer(&ptr->current);

    init_signal(&ptr->signal);

    // Start up the background worker thread.
    start_background_thread(ptr);

    return ptr;
}

int start_background_thread(ServerT *handle)
{
    pthread_t id;

    // Launch the background thread.
    if (pthread_create(&id, NULL, background_thread, (void *)handle) != 0)
    {
        return -1;
    }

    // Detach the background thread.
    pthread_detach(id); // TODO: Check error.

    // Link the worker thread.
    handle->worker_thread = id;
}

/// @brief Closes the server and frees the resources.
/// @param handle the handle to the server.
/// @return if it is succesful
int close_server(ServerT *handle)
{

    // Commit to the background thread.
    commit_to_background_thread(handle);

    // Free the allocation by the server.
    free((void *)handle);
    return 1;
}

/// @brief Checks if the server is locked.
/// @param handle the handle to the orderbook server.
/// @return if the handle is locked or not.
int check_locked(ServerT *handle)
{
    return handle->ctrl != 0;
}



/// @brief Commits records to file.
/// @param handle
/// @return
int commit_to_file(ServerT *handle)
{
    // If we are locked then report we are busy.
    if (check_locked(handle))
    {
        errno = EBUSY;
        return -1;
    }
    // Not done.
    return 1;
}

int try_lock(ServerT *handle, __uint32_t claimant)
{
    if (handle->ctrl == 2)
    {
        errno = EBUSY;
        return 0;
    }

    // We indicate that we are currently doing stuff with the dataabse.
    handle->ctrl = 1;

    for (int i = 0; i < 10000; i++)
    {
        // doing some sort of lookup lmao.
    }

    // We are done and grant access
    // THIS INTRODUCES A VULNERABILITY
    handle->ctrl = 2;

    handle->user_id = claimant;

    return 1;
}

void release_lock(ServerT *handle, __uint32_t claimant)
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
__uint32_t fetch_current_user(ServerT *handle)
{
    return handle->user_id;
}

int open_record(ServerT *handle)
{

    if (handle->current_order.status != 0)
    {
        return -1; // We are currently processing an order.
    }

    // Set the current record to open.
    handle->current_order.status = 1;

    return 1;
}



int set_recipient(ServerT *handle, int id)
{
    if (handle->current_order.status != 1)
    {
        return -1; // We are not currently processing an order.
    }

    // Set the current record's recipient.
    handle->current_order.recipient = id;

    return 1;
}

int set_sender(ServerT *handle, int id)
{
    if (handle->current_order.status != 1)
    {
        return -1; // We are not currently processing an order.
    }

    // Set the current record's sender.
    handle->current_order.sender = id;

    return 1;
}

int set_money(ServerT *handle, int id)
{
    if (handle->current_order.status != 1)
    {
        return -1; // We are not currently processing an order.
    }

    // Set the current record's sender.
    handle->current_order.money = id;

    return 1;
}

int flush_order(ServerT *handle)
{
    if (buffer_pos(handle) == 15)
    {
        return -1; // no room to flush orders.
    }
    if (handle->current_order.status != 1)
    {
        return -1; // current order is not closed out.
    }
    printf("fushing orer %d\n", handle->current_order.money);
    buffer_push(&handle->current, handle->current_order);
    handle->current_order.status = 0;

    // printf("op: %d\n", handle->current.pos);

    print_buffer(&handle->current);


    wait_signal(&handle->signal, 0);
    transfer_buffers(&handle->current, &handle->background);
    set_signal(&handle->signal, 1);

    return 0;
}

void commit_to_background_thread(ServerT *handle) {
    // Wait for the background thread to be ready to accept new orders.
    wait_signal(&handle->signal, 0);

    //
    transfer_buffers(&handle->current, &handle->background);
    
    // Notify that we have records available.
    set_signal(&handle->signal, 1);
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

void log_last_order(ServerT *handle)
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