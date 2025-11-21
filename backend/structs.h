#ifndef STRUCTS_H
#define STRUCTS_H

#include <pthread.h>
#include <linux/futex.h>  /* Definition of FUTEX_* constants */
#include "signals/signal.h"

typedef struct order_t {
    int recipient;
    int sender;
    int money;
    /* Is the order writeable */
    int status;
} Order;



typedef struct buffer_t {   
    Order orders[16];
    int pos;
} Buffer;


typedef struct masterbook_t {
    Buffer working;
    Signal book_signal;
    pthread_t handle;
    _Atomic(int) should_die;
} MasterBook;



typedef struct server_t {
    /// @brief The ID of the order book, this is used to confirm opening i.
    int id;
    /// @brief The user ID currently accessing the book.
    __uint32_t user_id;
    int user; // this tells us what user is currently using the book.
    int ctrl; // this tells us if the order book is currently busy.
    Order current_order;
    /// @brief The buffer of the current front-end state of the order book.
    Buffer current;

    struct signal_t signal;
    /// @brief The buffer for the background writer.
    Buffer background;
    /// @brief The handle to the worker thread.
    pthread_t worker_thread;
    
    MasterBook *master;
} ServerT;

#endif