#include "master.h"
#include <stdlib.h>
#include <stdio.h>
#include <pthread.h>
#include <unistd.h>
#include <fcntl.h>
#include <string.h>
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


/// @brief Configures the database, writing the header if necessary.
/// @return The file descriptor of the database.
int preconfigure_database() {
    char *name = "database.csv";
    struct stat buffer;


    int did_exist = 0;
    // Let us first check if the database exists (i.e., do we need to write
    // the header file)
    if(stat(name, &buffer) == 0) {
        // The database already exists. We are already setup.
        did_exist = 1;
    } else {
        // We need to create the database since it does not exist.
        did_exist = 0;
    }


    // Create the database on disk.
    int fd = open(name, O_WRONLY | O_APPEND | O_CREAT, 0644);
    if(fd == -1) {
        return -1; // failed to open the database.
    }

    if(!did_exist) {
        // If the file did not exist we need to start by writing the header.
        char *header = "sender,recipient,money\n";
        int result = write(fd, (void *) header, sizeof(char) * strlen(header));
        if(result == -1) {
            close(fd); // Close the file and propagate the error.
            return -1;
        }
    }
    return fd;


}




/// @brief The executable for the background thread.
/// @param handle The handle to the master order book.
void background_thread(MasterBook *handle)
{
 
    // Setup the database on the disk, this just
    // creates it if it does not exist.
    int db_fd = preconfigure_database();
    if(db_fd == -1) {
        return -1; // Bubble the error up.
    }



    while (1)
    {
        MbMsg msg = pop_channel(&handle->chan_t);
        if(msg.tag == 0) {

            Order order = msg.msg.order;

            print_order2(order);
            printf("\n");

        } else if(msg.tag == 1) {
            // This is the shutdown message.
            break;
        }
    }

    // The cleanup code.
    printf("cleaning up...");
    
    // Close the database.
    close(db_fd);
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


    // switch_signal(&book->book_signal, SIGNAL_READY2, SIGNAL_EMPTY);
    return book;
}



/// @brief Pushes records to the Master Order Book
/// @param ptr the pointer to the master order book.
/// @param src the source buffer to empty
/// @return how many items were pushed.
int push_records(MasterBook *ptr, Buffer *src) {

    int items = src->pos;
    for(int i = 0; i < items; i++) {
        // Create a message.
        MbMsg msg;
        msg.tag = 0;
        msg.msg.order = src->orders[i];

        // Push it to the channel.
        push_channel(&ptr->chan_t, msg);
    }
    // Reset the buffer position.
    src->pos = 0;
    return items;
}



/// @brief Closes the master's server, this is when it is released.
/// @param ptr the ptr to the master server
/// @return if we succesfully closed the master server
int close_master_server(MasterBook *ptr) {
    // NOTE: You may be wondering, what happens to our file,
    // well, funny you should ask! It is managed by the background
    // thread.
    printf("Closing\n");

    // Send a kill message, which will cause the thread to shut down.
    MbMsg kill;
    kill.tag = 1;
    push_channel(&ptr->chan_t, kill);



    // Wait for the thread to join.
    if (pthread_join(ptr->handle, NULL) != 0) {
        return -1; // There was an error.
    }

    // Free the channel.
    destroy_channel(&ptr->chan_t);


    // Free the memory.
    free((void *) ptr);
    return 0;
}