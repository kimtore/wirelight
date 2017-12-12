/*
 * LEDServer by Kim Tore Jensen <https://github.com/ambientsound/wirelight>.
 *
 * This is the server component. It listens on an UDP socket for Protobuf messages.
 */

#include <signal.h>
#include <zmq.h>

#include "led.h"
#include "pb.pb-c.h"

#define ADDRESS "tcp://0.0.0.0:1230"

static uint8_t running = 1;

static void ctrl_c_handler(int signum)
{
    (void)(signum);
    running = 0;
}

static void setup_handlers(void)
{
    struct sigaction sa;
    sa.sa_handler = ctrl_c_handler;

    sigaction(SIGINT, &sa, NULL);
    sigaction(SIGTERM, &sa, NULL);
}

static LED * recv_zmq_led(void * socket)
{
    char buffer[64];
    int size = zmq_recv(socket, buffer, 64, 0);
    if (size == -1) {
        return NULL;
    }
    buffer[size] = '\0';
    return led__unpack(NULL, size, (const uint8_t *)buffer);
}

int main() {
    uint64_t count = 0;
    LED *    led;

    // set up signal handlers.
    setup_handlers();

    // set up ZeroMQ context and socket.
    void *context = zmq_ctx_new();
    void *subscriber = zmq_socket(context, ZMQ_SUB);
    int rc = zmq_bind(subscriber, ADDRESS);
    assert (rc == 0);

    // no filtering.
    rc = zmq_setsockopt(subscriber, ZMQ_SUBSCRIBE, "", 0);
    assert (rc == 0);

    // socket is set up; initialize the LEDs.
    ledstrip_init();
    ledstrip_clear();
    ledstrip_render();

    while (running) {
        led = recv_zmq_led(subscriber);
        if (led == NULL) {
            fprintf(stderr, "invalid protobuf message received; discarding.\n");
            continue;
        }
        ledstrip_assign(led->index, led->rgb);
        if (led->render) {
            ledstrip_render();
        }
        led__free_unpacked(led, NULL);
        ++count;
    }

    zmq_close(subscriber);
    zmq_ctx_destroy(context);

    ledstrip_clear();
    ledstrip_finish();

    printf("\nProcessed %llu LED updates.\n", count);

    return 0;
}
