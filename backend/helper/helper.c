#include "helper.h"
#include <string.h>



/** BUFFER STUFF */

/// @brief Initiaizes a buffer to a start state.
/// @param ptr The pointer to the buffer.
void init_buffer(Buffer *ptr)
{
    ptr->pos = 0;
}


/// @brief Gets the current buffer cursor.
/// @param ptr The pointer to the buffer.
/// @return Location of the buffer cursor.
int buffer_pos(Buffer *ptr)
{
    return ptr->pos;
}

/// @brief Checks if the buffer is full.
/// @param ptr The pointer to the buffer.
/// @return Location of the buffer cursor.
int buffer_full(Buffer *ptr)
{
    return buffer_pos(ptr) >= 16;
}

/// @brief Pushes to the buffer.
/// @param ptr The pointer to the buffer.
/// @param entry The entry to the buffer.
/// @return The status.
int buffer_push(Buffer *ptr, Order entry)
{
    if (buffer_full(ptr))
    {
        return -1;
    }

    ptr->orders[ptr->pos++] = entry;

    return 1;
}