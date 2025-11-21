#include "structs.h"
#include <stdio.h>
#include "helper/helper.h"


ServerT *open_server(int id);
int close_server(ServerT *ptr);
int open_record(ServerT *ptr);
int try_lock(ServerT *handle, __uint32_t claimant);
int set_recipient(ServerT *handle, int id);
int set_sender(ServerT *handle, int id);
int set_money(ServerT *handle, int money);
void log_last_order(ServerT *handle);
int flush_order(ServerT *handle);
void release_lock(ServerT *handle, __uint32_t id);
__uint32_t fetch_current_user(ServerT *handle);