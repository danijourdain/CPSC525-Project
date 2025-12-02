#ifndef STRUCTS_H
#define STRUCTS_H

#include <pthread.h>
#include <linux/futex.h>  /* Definition of FUTEX_* constants */
#include "signals/signal.h"
#include <stdio.h>

#define REGIONS 3

typedef struct order_t {
    int recipient;
    int sender;
    int money;
    int region;
    /* Is the order writeable */
    int status;
} Order;




typedef struct buffer_t {   
    Order orders[16];
    int pos;
} Buffer;

typedef enum {
    MSG_CLOSE,
    MSG_ORDER
} MessageKind;

typedef struct mbmsg_t {
    MessageKind tag;
    union {
        Order order;
    } msg;
} MbMsg;

typedef struct chan_node_t {
    MbMsg contents;
    struct chan_node_t *next;
} ChanNode;



typedef struct chan_t {
    ChanNode *head;
    pthread_mutex_t mutex;
    Signal sig;
} Channel;



typedef struct order_list_t {
    Order *list;    
    int capacity;
    int length;
} OrderList;

typedef struct masterbook_t {
    Buffer working;
    Signal book_signal;
    Channel chan_t;
    FILE *ledger_fd;
    OrderList order_list;
    pthread_t handle;
    int balances[REGIONS];
    pthread_mutex_t balance_mutex;
    _Atomic(int) should_die;
} MasterBook;


typedef enum {
    SEC_NONE,
    SEC_LOW,
    SEC_MID,
    SEC_HIGH,
    SEC_VERYHIGH
} SecLevel;



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
    SecLevel security_level;

    // struct signal_t signal;
    /// @brief The buffer for the background writer.
    Buffer background;
    /// @brief The handle to the worker thread.
    pthread_t worker_thread;
    
    pthread_mutex_t fixer_tex;
    MasterBook *master;

    long req_count;
} SubjugateOrderBook;

#endif