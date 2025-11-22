#ifndef MASTER_H
#define MASTER_H

#include "../structs.h"



MasterBook *open_master_server();
int close_master_server(MasterBook *ptr);
int push_records(MasterBook *ptr, Buffer *src);
int query_regions();
int get_balance(MasterBook *ptr, int region);
char *get_region_name(int id);
char *get_region_password(int id);


#endif