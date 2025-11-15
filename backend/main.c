
#include <stdlib.h>
#include <stdio.h>
#include "structs.h"
#include <errno.h>




ServerT *open_server(int id) {
    ServerT *ptr = (ServerT *) malloc(sizeof(ServerT));
    ptr->id = id;
    ptr->current_order.status = 0;
    ptr->buffer_pos = 0;
    return ptr;
}



int try_lock(ServerT *handle, __uint32_t claimant) {
    if(handle->ctrl == 2) {
        errno = EBUSY;
        return 0;
    }
    
    // We indicate that we are currently doing stuff with the dataabse.
    handle->ctrl = 1;
    
    for(int i = 0; i < 10000; i++) {
        // doing some sort of lookup lmao.
    }

    // We are done and grant access
    // THIS INTRODUCES A VULNERABILITY
    handle->ctrl = 2;

    handle->user_id = claimant;
    
    return 1;
}

void release_lock(ServerT *handle, __uint32_t claimant) {
    if(handle->user_id == claimant) {
        handle->ctrl = 0;
        handle->user_id = 0;
    }
}

__uint32_t fetch_current_user(ServerT *handle) {
    return handle->user_id;
}

int open_record(ServerT *handle) {

    

    if(handle->current_order.status != 0) {
        return -1; // We are currently processing an order.
    }

    // Set the current record to open.
    handle->current_order.status = 1;

    return 1;
}

int set_recipient(ServerT *handle, int id) {
    if(handle->current_order.status != 1) {
        return -1; // We are not currently processing an order.
    }

    // Set the current record's recipient.
    handle->current_order.recipient = id;

    return 1;
}

int set_sender(ServerT *handle, int id) {
    if(handle->current_order.status != 1) {
        return -1; // We are not currently processing an order.
    }

    // Set the current record's sender.
    handle->current_order.sender = id;

    return 1;
}

int set_money(ServerT *handle, int id) {
    if(handle->current_order.status != 1) {
        return -1; // We are not currently processing an order.
    }

    // Set the current record's sender.
    handle->current_order.money = id;

    return 1;
}

int flush_order(ServerT *handle) {
    if(handle->buffer_pos == 15) {
        return -1; // no room to flush orders.
    }
    if(handle->current_order.status != 1) {
        return -1; // current order is not closed out.
    }
    handle->order_buffer[handle->buffer_pos] = handle->current_order;
    handle->buffer_pos++;
    handle->current_order.status = 0;
    return 0;
}


int log_last_order(ServerT *handle) {
    if(handle->buffer_pos == 0) {
        printf("log: nothing to show.\n");
    } else {
        Order entry = handle->order_buffer[handle->buffer_pos];
        printf("log: %d -> %d ($%d)\n", entry.sender, entry.recipient, entry.money);
    }
}