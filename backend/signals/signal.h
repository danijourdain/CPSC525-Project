
#ifndef SIGNALS_H
#define SIGNALS_H

#include <stdatomic.h>

typedef struct signal_t {
    int state;
    volatile _Atomic(int) lock;
} Signal;


void set_signal(Signal *sg, int state);
void wait_signal(Signal *sgnl, int state);
void init_signal(Signal *sgnl);

#endif