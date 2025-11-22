#include "channel.h"
#include <pthread.h>
#include <stdlib.h>


/// @brief Initializes a new channel with no elements.
/// @param chan The pointer to the channel to intialize.
void init_channel(Channel *chan) {
    chan->head = NULL;
    pthread_mutex_init(&chan->mutex, NULL);
    init_signal(&chan->sig);
}

/// @brief Destroys the channel.
/// @param chan The pointer to the channel.
void destroy_channel(Channel *chan) {
    // Lock the mutex.
    pthread_mutex_lock(&chan->mutex);


    if(chan->head != NULL) {
        // If the channel has elements in it, then
        // we deallocate all of those elements.
        ChanNode *current = chan->head;
        while(current->next != NULL) {
            ChanNode *next = current->next;
            free((void *) current);
            current = next;
        }
        // Free the current entry.
        free((void *) current);
    }


    // Destroy the mutex.
    pthread_mutex_destroy(&chan->mutex);

}

/// @brief Makes a new channel node with contents.
/// @param msg The message to store within the node.
/// @return The channel node.
ChanNode *make_node(MbMsg msg) {
    ChanNode *node = (ChanNode *) malloc(sizeof(ChanNode));
    node->next = NULL;
    node->contents = msg;
    return node;
}

/// @brief Pushes a new entry to the channel.
/// @param chan The pointer to the channel.
/// @param msg The message to send.
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


    // Now we have an item in the channel.
    set_signal_immediate(&chan->sig, 1);

    // Unlock the mutex.
    pthread_mutex_unlock(&chan->mutex);
}



/// @brief Pops from the channel.
/// @param chan The pointer to the channel.
/// @return The message that was popped.
MbMsg pop_channel(Channel *chan) {
    while(1) {
        // Wait for there to be items in the channel.
        wait_signal(&chan->sig, 1);
        // Lock the mutex.
        pthread_mutex_lock(&chan->mutex);

        
        if(chan->head != NULL) {
            // If there is ACTUALLY an item.
            ChanNode *next = chan->head->next;

            // Extract the payload by copying it out.
            MbMsg payload = chan->head->contents;

     
            // Now we free the current head.
            free((void *) chan->head);


            // Set the head to the next item.
            chan->head = next;

            if(next == NULL) {
                // Lower the signal.
                set_signal_immediate(&chan->sig, 0);
            }
        
            // Unlock the mutex and return early.
            pthread_mutex_unlock(&chan->mutex);
            return payload;
        }


        // Unlock the mutex.
        pthread_mutex_unlock(&chan->mutex);
    }
    



}