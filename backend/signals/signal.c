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
    atomic_store(&sgnl->state, 0);
    // sgnl->state = 0;
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


/// @brief Wakes up a futex at an atomic address
/// @param addr the address of the atomic to wake up on
/// @param n number to wake
/// @return the ones woken up
static int futex_wake(_Atomic volatile int *addr, int n)
{
    return syscall(SYS_futex,
                   addr,
                   FUTEX_WAKE,
                   n,
                   NULL,
                   NULL,
                   0);
}


/// @brief Raises a signal immediately without waiting
/// @param sg The signal to use.
/// @param state The state to raise to.
void set_signal_immediate(Signal *sg, int state)
{
    atomic_store(&sg->lock, state);
    futex_wake(&sg->lock, INT_MAX);
}

/// @brief The signal to wait for.
/// @param sgnl The signal to wait on.
/// @param state The state to wait for.
void wait_signal(Signal *sgnl, int state)
{
    while (1)
    {
        // Check if the value is set.
        if (atomic_load(&sgnl->lock) == state)
        {
            return;
        }

        // Wait for a wakeup.
        futex_wait(&sgnl->lock, state);
    }
}

/// @brief This is a wait and switch primitive.
/// @param sg The signal to wait & switch on.
/// @param start The start state.
/// @param state The end state.
void switch_signal(Signal *sg, int start, int state) {
    while(1)
    {
        int expected = start;

        // Do a CMPXNCHG to switch over the states.
        if(atomic_compare_exchange_strong(&sg->lock, &expected, state) && expected == start) {
            // Make sure to notify of succesful changeover.
            futex_wake(&sg->lock, INT_MAX);
            return;
        }
        // Wait on the futex to terminate.
        futex_wait(&sg->lock, start);
    }
}