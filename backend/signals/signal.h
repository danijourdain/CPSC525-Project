
#ifndef SIGNALS_H
#define SIGNALS_H

#include <stdatomic.h>

#define SIGNAL_EMPTY 0
#define SIGNAL_LOCKED 1
#define SIGNAL_READY 2
#define SIGNAL_READY2 3
#define SIGNAL_LOCKED2 4

typedef struct signal_t {
    _Atomic(int) state;
    volatile _Atomic(int) lock;
} Signal;

void set_signal_immediate(Signal *sg, int state);
void switch_signal(Signal *sg, int start, int state);
void wait_signal(Signal *sgnl, int state);
void init_signal(Signal *sgnl);

#endif