#ifndef HELPER_H
#define HELPER_H

#include "../structs.h"


void init_buffer(Buffer *ptr);
int buffer_pos(Buffer *ptr);
int buffer_full(Buffer *ptr);
int buffer_push(Buffer *ptr, Order entry);

#endif