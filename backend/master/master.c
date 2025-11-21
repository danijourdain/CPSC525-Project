#include "master.h"
#include <stdlib.h>

MasterBook *open_master_server() {
    MasterBook *book = (MasterBook *) malloc(sizeof(MasterBook));

    init_signal(&book->book_signal);

    return book;


}