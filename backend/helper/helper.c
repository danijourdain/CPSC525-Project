#include "helper.h"
#include <string.h>

/// @brief Initiaizes a buffer to a start state.
/// @param ptr
void init_buffer(Buffer *ptr)
{
    ptr->pos = 0;
}

/// @brief Transfers the contents of the source buffer to the destination buffer.
/// @param src The source buffer to copy from.
/// @param dst The destinatn
/// @returns the number of items transfere.
int transfer_buffers(Buffer *src, Buffer *dst)
{
    if (buffer_pos(src) == 0 || buffer_pos(dst) >= 16)
    {
        return 1;
    }

    // The number of items to transfer.
    int items = buffer_pos(src);

    // The room left in the transfer buffer.
    int space = 16 - buffer_pos(dst);

    int to_transfer = 0;
    if (items < space)
    {
        // We can transfer all the items.
        to_transfer = items;
    }
    else
    {
        return -1; // To simplify this code we do not allow this as it would require shifting.
    }

    // Copy over the memory to the new buffer.
    memcpy((void *)dst->orders, (void *)src->orders, (to_transfer * sizeof(Order)));

    // Change the positions.
    src->pos = 0;
    dst->pos = to_transfer;

    return 1;
}

/// @brief
/// @param ptr
/// @return
int buffer_pos(Buffer *ptr)
{
    return ptr->pos;
}

int buffer_full(Buffer *ptr)
{
    return buffer_pos(ptr) >= 16;
}

int buffer_push(Buffer *ptr, Order entry)
{
    if (buffer_full(ptr))
    {
        return -1;
    }

    ptr->orders[ptr->pos++] = entry;

    return 1;
}