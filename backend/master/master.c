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
#include <error.h>
#include <errno.h>
#include <limits.h>
#include <inttypes.h>


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


/// @brief Reads a field from a CSV row. Inspired by https://stackoverflow.com/questions/12911299/read-csv-file-in-c.
/// @param src The source line.
/// @param n The field to access.
/// @return The text.
char *read_field(char *src, int n) {
    const char* tok;
    for (tok = strtok(src, ",");
            tok && *tok;
            tok = strtok(NULL, ",\n"))
    {
        if (!--n)
            return tok;
    }
    return NULL;
}


/// @brief Loads the ledger file from a name.
/// @param name The name of the ledger file.
/// @return The file descriptor.
int load_ledger_file(char *name) {
    struct stat buffer;


    int did_exist = 0;
    // Let us first check if the database exists (i.e., do we need to write
    // the header file)
    if(stat(name, &buffer) != 0) {
        return -1; // Ledger did not exist.
    }


    // Create the database on disk.
    int fd = open(name, O_RDWR, 0644);
    if(fd == -1) {
        return -1; // failed to open the database.
    }

    
    return fd;
}


// int read_out_ledger(int *balances, FILE *fp) {

//     int i = 0;
//     char line[1024];
//     while(fgets(line, sizeof(line), fp)) {
        
//         if(i == 0 && strncmp("region,balance\n", line, sizeof(line)) != 0) {
//             // Make sure that the first line is properly formatted.
//             return -1;
//         } else if(i > 0) {
//             // Extract the CSV lines.
//             char *region_balance_str = read_field(line, 2);
//             char *region_id_str = read_field(line, 1);
//             if(region_id_str == NULL || region_balance_str == NULL) {
//                 return -1; // One of the pointers is NULL.
//             }

//             // Parse the line.
//             // The following was helpful: https://stackoverflow.com/questions/7021725/how-to-convert-a-string-to-integer-in-c
//             uintmax_t num = strtoumax(region_id_str, NULL, 10);
//             if (num == UINTMAX_MAX && errno == ERANGE) {
//                 return -1; 
//             }

//             if(num >= REGIONS) {
//                 return -1; // Not a valid region ID.
//             }


//             uintmax_t balance = strtoumax(region_balance_str, NULL, 10);
//             if(balance == UINTMAX_MAX && errno == ERANGE) {
//                 return -1; // Failed to parse.
//             }

//             balances[num] = (int) balance;



//         }

       

//         // Increment the control.
//         i += 1;
//     }
//     return 0;

// }

int load_database(char *name, MasterBook *book) {
    // Let us start by opening the file.
    FILE *fd = fopen(name, "r");
    if(fd == NULL) {
        return -1;
    }


    OrderList *list = &book->order_list;
    

    pthread_mutex_lock(&book->balance_mutex);
    int i = 0;
    char line[1024];
    while(fgets(line, sizeof(line), fd)) {
        
        if(i == 0 && strncmp("sender,recipient,money\n", line, sizeof(line)) != 0) {
            // Make sure that the first line is properly formatted.
            pthread_mutex_unlock(&book->balance_mutex);
            return -1;
        } else if(i>0) {
            char line_restore[1024];

            // Restore and read.
            strncpy(line_restore, line, sizeof(line));
            char *money = read_field(line_restore, 3);

            // Restore and read.
            strncpy(line_restore, line, sizeof(line));
            char *sender = read_field(line_restore, 1);

            // Restore and read.
            strncpy(line_restore, line, sizeof(line));
            char *recipient = read_field(line_restore, 2);
            
            
            
            // Make sure that nothing is null.
            if(money == NULL || recipient == NULL || sender == NULL) {
                pthread_mutex_unlock(&book->balance_mutex);
                return -1; // The fields are null;
            }

         
            
            // Parse the line.
            // The following was helpful: https://stackoverflow.com/questions/7021725/how-to-convert-a-string-to-integer-in-c
            uintmax_t money_num = strtoumax(money, NULL, 10);
            if (money_num == UINTMAX_MAX && errno == ERANGE) {
                pthread_mutex_unlock(&book->balance_mutex);
                return -1; 
            }
            

            // Parse the line.
            // The following was helpful: https://stackoverflow.com/questions/7021725/how-to-convert-a-string-to-integer-in-c
            uintmax_t sender_num = strtoumax(sender, NULL, 10);
            if (sender_num == UINTMAX_MAX && errno == ERANGE) {
                pthread_mutex_unlock(&book->balance_mutex);
                return -1; 
            }

             // Parse the line.
            // The following was helpful: https://stackoverflow.com/questions/7021725/how-to-convert-a-string-to-integer-in-c
            uintmax_t recipient_num = strtoumax(recipient, NULL, 10);
            if (recipient_num == UINTMAX_MAX && errno == ERANGE) {
                pthread_mutex_unlock(&book->balance_mutex);
                return -1; 
            }

            // printf("Found order: %d, %d, %d\n", sender_num, recipient_num, money_num);

            if(list->length == list->capacity) {
                list->capacity += 20;
                list->list = realloc(list->list, sizeof(Order) * list->capacity);
            }

            Order order;
            order.region = sender_num;
            order.status = 0;
            order.recipient = recipient_num;
            order.money = money_num;
            order.sender = sender_num;
            list->list[list->length++] = order; 

            // printf("List Length: %d\n", list->length);
            
        }
        i++;
    }

    pthread_mutex_unlock(&book->balance_mutex);


    // Close the file descriptor.
    fclose(fd);

    // We return with no errors.
    return 0;
}

int get_database_length(MasterBook *book) {

    pthread_mutex_lock(&book->balance_mutex);
    int length = book->order_list.length;
    pthread_mutex_unlock(&book->balance_mutex);
    return length;
}

Order get_database_entry_at(MasterBook *book, int position) {
    pthread_mutex_lock(&book->balance_mutex);
    Order order = book->order_list.list[position];
    pthread_mutex_unlock(&book->balance_mutex);

    return order;
}
 

/// @brief Configures the database, writing the header if necessary.
/// @return The file descriptor of the database.
int preconfigure_database(MasterBook *book) {
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


    if(did_exist) {
        printf("log: database already exists. loading records.\n");
        if(load_database(name, book) == -1) {
            // free((void *) orderListlist);
            return -1; // Bubble up the error.
        }

        printf("log: loaded database. (%d)\n", book->order_list.length);
    }


    // Create the database on disk.
    int fd = open(name, O_WRONLY | O_APPEND | O_CREAT, 0644);
    if(fd == -1) {
        // free((void *) orderList.list);
        return -1; // failed to open the database.
    }

    if(!did_exist) {
        // If the file did not exist we need to start by writing the header.
        char *header = "sender,recipient,money\n";
        int result = write(fd, (void *) header, sizeof(char) * strlen(header));
        if(result == -1) {
            // free((void *) orderList.list);
            close(fd); // Close the file and propagate the error.
            return -1;
        }
    }
    return fd;


}


/// @brief Writes an order to the master book.
/// @param fd the file descriptor to write to.
/// @param order The order to write.
/// @return The status code.
int write_order(int fd, Order order) {



    // Format the string that will be written to the database.
    char buf[1024];
    int status = snprintf(buf, sizeof(buf), "%d,%d,%d\n", order.region, order.recipient, order.money);
    if(status == 0 || status >= (int) sizeof(buf)) {
        return -1; // Did not write properly.
    }

    int result = write(fd, (void *) buf, status);
    if(result == -1) {
        return -1; // Bubble up the error. The master code should handle this.
    }


    // We were able to write the order succesfully.
    return 0; 
}



/// @brief Writes back to the ledger.
/// @param handle The pointer to the master book.
/// @return the status
int ledger_writeback(MasterBook *handle) {
    fflush(handle->ledger_fd);
    // Truncate the file.
    int res = ftruncate(fileno(handle->ledger_fd), 0);
    if(res == -1) {
        perror("failed to truncate the file.");
        return -1;
    }

    fseek(handle->ledger_fd, 0, 0);

    // Write the header to the file.
    char *header = "region,balance\n";
    if(fwrite(header, sizeof(char), 15, handle->ledger_fd) == 0) {
        perror("failed to write header.");
        return -1;
    }

    // Now write the CSV rows.
    for(int i = 0; i < REGIONS; i++) {
        if(fprintf(handle->ledger_fd, "%d,%d\n", i, handle->balances[i]) == 0) {
            perror("failed to write...,");
            return -1;
        }
    }


    // Flush the file contents.
    fflush(handle->ledger_fd);
}


/// @brief Gets the balance of a particular region.
/// @param handle The handle.
/// @param region The particular region we are interested in.
/// @return Returns a balance.
int get_balance(MasterBook *handle, int region) {
    if(region > 0 || region > REGIONS) {
        return -1;
    }
    pthread_mutex_lock(&handle->balance_mutex);
    int value = handle->balances[region];
    pthread_mutex_unlock(&handle->balance_mutex);
    return value;
}

/// @brief The executable for the background thread.
/// @param handle The handle to the master order book.
void background_thread(MasterBook *handle)
{
 
    // Setup the database on the disk, this just
    // creates it if it does not exist.
    int db_fd = preconfigure_database(handle);
    if(db_fd == -1) {
        // free((void *))
        return; // Bubble the error up.
    }

    while (1)
    {
        MbMsg msg = pop_channel(&handle->chan_t);
        if(msg.tag == MSG_ORDER) {

            Order order = msg.msg.order;


            if(order.money > handle->balances[order.sender]) {
                continue;
            }

            

            // Write the order to a file, recalling
            // that we have mutually exclusive access to the file
            // at this point in time.
            //
            // NOTE: We want to pass on failed orders so
            // we ignore errors here.
            write_order(db_fd, order);


      
            // Perform the money transfer.
            pthread_mutex_lock(&handle->balance_mutex);
            
            // Add the order to the list.
            OrderList *list = &handle->order_list;
            if(list->length == list->capacity) {
                list->capacity += 20;
                list->list = realloc(list->list, sizeof(Order) * list->capacity);
            }
            list->list[list->length++] = order; 


            handle->balances[order.sender] -= order.money;
            handle->balances[order.recipient] += order.money;
            

            // Perform a writeback, we ignore errors here.
            ledger_writeback(handle);
            pthread_mutex_unlock(&handle->balance_mutex);

        } else if(msg.tag == MSG_CLOSE) {
            // This is the shutdown message.
            break;
        }
    }


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



/// @brief Gets the password for a region hashed once.
/// @param id The ID of the region.
/// @return The password for the region hashed once.
char *get_region_password(int id) {
    if(id == 0) {
        // Calgary
        return "8757871d465a13613ab3f863e44cc31fd5efa25c02357b154e5ae8fe560c1d54";
    } else if(id == 1) {
        // New York
        return "18d5a3ce8b6ef9b4b4a7e9e32edd750b3135918f02c0249b5d76c6ad9b19da96";
    } else if(id == 2) {
        // Signapore
        return "a6e3870ad1cc954d4a71fcf23455367b7fdafe1a0c0d3a9666991c3438b3200b";
    } else {
        // No region could be found.
        return NULL;
    }
}



int read_out_ledger(int *balances, FILE *fp) {

    int i = 0;
    char line[1024];
    while(fgets(line, sizeof(line), fp)) {
        
        if(i == 0 && strncmp("region,balance\n", line, sizeof(line)) != 0) {
            // Make sure that the first line is properly formatted.
            return -1;
        } else if(i > 0) {
            // Extract the CSV lines.
            char *region_balance_str = read_field(line, 2);
            char *region_id_str = read_field(line, 1);
            if(region_id_str == NULL || region_balance_str == NULL) {
                return -1; // One of the pointers is NULL.
            }

            // Parse the line.
            // The following was helpful: https://stackoverflow.com/questions/7021725/how-to-convert-a-string-to-integer-in-c
            uintmax_t num = strtoumax(region_id_str, NULL, 10);
            if (num == UINTMAX_MAX && errno == ERANGE) {
                return -1; 
            }

            if(num >= REGIONS) {
                return -1; // Not a valid region ID.
            }


            uintmax_t balance = strtoumax(region_balance_str, NULL, 10);
            if(balance == UINTMAX_MAX && errno == ERANGE) {
                return -1; // Failed to parse.
            }

            balances[num] = (int) balance;



        }

       

        // Increment the control.
        i += 1;
    }
    return 0;

}


/// @brief Opens a new master server.
/// @return the pointer to the master server.
MasterBook *open_master_server() {
    MasterBook *book = (MasterBook *) malloc(sizeof(MasterBook));


    // Open the ledger.
    int res = load_ledger_file("ledger.csv");
    if(res == -1) {
        perror("could not find the ledger.csv file.\n");
        return NULL;
    }

    // Open the ledger as a file pointer.
    FILE *ledger_fd = fdopen(res, "r+");
    if(ledger_fd == NULL) {
        close(res); // Try to close it normally;
        return NULL;
    }


    // Read out the ledger.
    if(read_out_ledger(book->balances, ledger_fd) == -1) {
        perror("ledger was malformed.\n");
        close(res); // Close out the ledger file.
        return NULL;
    }


    // Set the ledger file descriptor.
    book->ledger_fd = ledger_fd;
   

    book->order_list.capacity = 20;
    book->order_list.list = (Order *) malloc(sizeof(Order) * book->order_list.capacity);
    book->order_list.length = 0;
    

    // Initialize the mutex.
    pthread_mutex_init(&book->balance_mutex, NULL);
       

    // log the balances.
    printf("log: balances read (");
    for(int i = 0; i < REGIONS; i++) {
        printf("%s: %d", get_region_name(i), book->balances[i]);
        if(i != REGIONS - 1) {
            printf(", ");
        }
    }
    printf(")\n");

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
        msg.tag = MSG_ORDER;
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

    // Send a kill message, which will cause the thread to shut down.
    MbMsg kill;
    kill.tag = MSG_CLOSE;
    push_channel(&ptr->chan_t, kill);



    // Wait for the thread to join.
    if (pthread_join(ptr->handle, NULL) != 0) {
        return -1; // There was an error.
    }


    // Free the order list.
    free((void *) ptr->order_list.list);

    // Free the channel.
    // This is safe because it waits for the Mutex to be
    // locked + we have already waited for the background
    // worker to terminate.
    destroy_channel(&ptr->chan_t);


    // Finally we free up the ledger.
    fclose(ptr->ledger_fd);

    // Destroy the mutex.
    pthread_mutex_destroy(&ptr->balance_mutex);


    // Free the memory.
    free((void *) ptr);
    return 0;
}