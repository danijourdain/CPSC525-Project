#include "structs.h"
#include <stdio.h>
#include "helper/helper.h"


SubjugateOrderBook *open_server(int id, MasterBook *master);
int close_server(SubjugateOrderBook *ptr);
int open_record(SubjugateOrderBook *ptr);
int try_lock(SubjugateOrderBook *handle, char *password);
int set_recipient(SubjugateOrderBook *handle, int id);
int set_sender(SubjugateOrderBook *handle, int id);
int set_money(SubjugateOrderBook *handle, int money);
void log_last_order(SubjugateOrderBook *handle);
int flush_order(SubjugateOrderBook *handle);
void release_lock(SubjugateOrderBook *handle, __uint32_t id);
__uint32_t fetch_current_user(SubjugateOrderBook *handle);