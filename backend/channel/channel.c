#include "channel.h"
#include <pthread.h>
#include <stdlib.h>

void init_channel(Channel *chan) {
    chan->head = NULL;
    pthread_mutex_init(&chan->mutex, NULL);
    init_signal(&chan->sig);
}

ChanNode *make_node(MbMsg msg) {
    ChanNode *node = (ChanNode *) malloc(sizeof(ChanNode));
    node->next = NULL;
    node->contents = msg;
    return node;
}


void push_channel(Channel *chan, MbMsg msg) {
    
    // We acquire the lock.
    pthread_mutex_lock(&chan->mutex);

    if(chan->head == NULL) {
        ChanNode *new = make_node(msg);
        chan->head = new;
    } else {
        ChanNode *current = chan->head;
        while(current->next != NULL) {
            current = current->next;
        }
        current->next = make_node(msg);
    }


    // Unlock the mutex.
    pthread_mutex_unlock(&chan->mutex);

    set_signal_immediate(&chan->sig, 1);
}



MbMsg pop_channel(Channel *chan) {
    wait_signal(&chan->sig, 1);

}