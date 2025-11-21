#include <stdio.h>
#include <linux/futex.h> /* Definition of FUTEX_* constants */
#include <sys/syscall.h> /* Definition of SYS_* constants */
#include <limits.h>
#include <stdlib.h>
#include <unistd.h>
#include "signal.h"



/// @brief Initializes a new signal.
/// @param sgnl The signal to initialize.
void init_signal(Signal *sgnl)
{
    atomic_store(&sgnl->lock, 0);
}

/// @brief Wakes up the futex.
/// @param addr 
/// @param expected 
/// @return 
static int futex_wait(_Atomic volatile int *addr, int expected)
{
    return syscall(
        SYS_futex,
        addr,
        FUTEX_WAIT,
        expected,
        NULL,
        NULL,
        0);
}

static int futex_wake(_Atomic volatile int *addr, int n)
{
    // Wake up to n waiters
    return syscall(SYS_futex,
                   addr,
                   FUTEX_WAKE,
                   n,
                   NULL,
                   NULL,
                   0);
}

void set_signal(Signal *sg, int state)
{
    atomic_store(&sg->lock, state);
    futex_wake(&sg->lock, INT_MAX);
}

void wait_signal(Signal *sgnl, int state)
{
    while (1)
    {
        if (atomic_load(&sgnl->lock) == state)
        {
            return;
        }
        futex_wait(&sgnl->lock, state);
    }
}
