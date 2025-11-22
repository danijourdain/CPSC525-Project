#ifndef CHANNEL_H
#define CHANNEL_H

#include "../structs.h"

void init_channel(Channel *chan);
void destroy_channel(Channel *chan);
void push_channel(Channel *chan, MbMsg msg);
MbMsg pop_channel(Channel *chan);

#endif