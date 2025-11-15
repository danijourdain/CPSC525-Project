

typedef struct order_t {
    int recipient;
    int sender;
    int money;
    /* Is the order writeable */
    int status;
} Order;

typedef struct server_t {
    int id;
    __uint32_t user_id;
    int user; // this tells us what user is currently using the book.
    int ctrl; // this tells us if the order book is currently busy.
    Order current_order;
    Order order_buffer[16];
    int buffer_pos;
} ServerT;