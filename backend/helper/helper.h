#ifndef HELPER_H
#define HELPER_H

#include "../structs.h"

/// @brief Initiaizes a buffer to a start state.
/// @param ptr
void init_buffer(Buffer *ptr);

int transfer_buffers(Buffer *src, Buffer *dst);

/// @brief
/// @param ptr
/// @return
int buffer_pos(Buffer *ptr);

int buffer_full(Buffer *ptr);

int buffer_push(Buffer *ptr, Order entry);

#endif