#include "master.h"
#include <stdlib.h>
#include <stdio.h>
#include <pthread.h>
#include "../helper/helper.h"
#include <sys/stat.h>
#include "../channel/channel.h"

void print_order2(Order ptr)
{
    printf("Order { sender: %d, recipient: %d, money: %d }", ptr.sender, ptr.recipient, ptr.money);
}

void print_buffer2(Buffer *src)
{
    printf("Buffer { pos: %d, buffer: [", src->pos);

    for (int i = 0; i < src->pos; i++)
    {
        print_order2(src->orders[i]);
        if (i != src->pos - 1)
        {
            printf(", ");
        }
    }

    printf("]}\n");
}


/// @brief Queries how many regions the master supports.
/// @returns the number of regions.
int query_regions() {
    return 3;
}


/// @brief Gets the name of the region from an ID.
/// @param id the region id
/// @return a ptr to a string or NULL if we could not retrieve the region name.
char *get_region_name(int id) {
    if(id == 0) {
        return "Calgary";
    } else if(id == 1) {
        return "New York";
    } else if(id == 2) {
        return "Signapore";
    } else {
        return NULL;
    }
}


void setup_database() {
    char *name = "database.csv";
    struct stat buffer;


    // Let us first check if the database exists (i.e., do we need to write
    // the header file)
    if(stat(name, &buffer) == 0) {
        // The database already exists. We are already setup.
    } else {
        // We need to create the database since it does not exist.
    }
}



void background_thread(MasterBook *handle)
{


    // Notify the main thread that we are setup.
    switch_signal(&handle->book_signal, SIGNAL_LOCKED2, SIGNAL_READY2);

    while (1)
    {
        switch_signal(&handle->book_signal, SIGNAL_READY, SIGNAL_LOCKED2);
        if(atomic_load(&handle->should_die)) {
            // In this case we were told to die.
            break;
        }
        printf("Received a packet.\n");

        print_buffer2(&handle->working);

        switch_signal(&handle->book_signal, SIGNAL_LOCKED2, SIGNAL_EMPTY);
    }

    // The cleanup code.
    printf("cleaning up...");
}


/// @brief Start the background thread that handles the master.
/// @param handle the handle to the background thread.
/// @return the status of if the background thread was created.
int start_background_thread(MasterBook *handle)
{
    pthread_t id;

    // Launch the background thread.
    if (pthread_create(&id, NULL, (void *) background_thread, (void *)handle) != 0)
    {
        return -1;
    }

    // Link the worker thread.
    handle->handle = id;

    return 0;
}


/// @brief Opens a new master server.
/// @return the pointer to the master server.
MasterBook *open_master_server() {
    MasterBook *book = (MasterBook *) malloc(sizeof(MasterBook));


    // The channel.
    init_channel(&book->chan_t);

    // Initialize the signal within.
    init_signal(&book->book_signal);
    set_signal_immediate(&book->book_signal, SIGNAL_LOCKED2);
    
    // Make sure we mark the book as live.
    atomic_store(&book->should_die, 0);


    // Start up the background worker.
    start_background_thread(book);


    switch_signal(&book->book_signal, SIGNAL_READY2, SIGNAL_EMPTY);
    return book;
}



int push_records(MasterBook *ptr, Buffer *src) {
    printf("pushing recordsss...\n");
    // Raise it to the locked status.
    switch_signal(&ptr->book_signal, SIGNAL_EMPTY, SIGNAL_LOCKED);

    // Write records.
    int items = transfer_buffers(src, &ptr->working);


    // Raise the lock to ready so that the background thread can acquire.
    switch_signal(&ptr->book_signal, SIGNAL_LOCKED, SIGNAL_READY);
    return items;
}



/// @brief Closes the master's server, this is when it is released.
/// @param ptr the ptr to the master server
/// @return if we succesfully closed the master server
int close_master_server(MasterBook *ptr) {
    printf("Closing\n");


    
    printf("Waiting..\n");
    // We force the signal up, closing out the order book.
    atomic_store(&ptr->should_die, 1);
    set_signal_immediate(&ptr->book_signal, SIGNAL_READY);


    printf("bru\n");

    

    // Wait for the thread to join.
    if (pthread_join(ptr->handle, NULL) != 0) {
        return -1; // There was an error.
    }



    // Free the memory.
    free((void *) ptr);
    return 0;
}