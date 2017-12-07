/*
 * LEDServer by Kim Tore Jensen <https://github.com/ambientsound/wirelight>.
 *
 * This is the server component. It listens on an UDP socket for Protobuf messages.
 */

#include <errno.h>
#include <netdb.h>
#include <netinet/in.h>
#include <signal.h>
#include <stdarg.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/socket.h>
#include <unistd.h>

#include "led.h"
#include "pb.pb-c.h"

#define HOST 0
#define PORT "1230"

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

int main() {
    int                     fd;
    int                     err;
    struct addrinfo         hints;
    struct addrinfo *       ai = 0;
    char                    buffer[548];
    struct sockaddr_storage src_addr;
    socklen_t               src_addr_len = sizeof(src_addr);
    ssize_t                 received;
    uint64_t                count = 0;
    LED *                   led;

    // create socket options for an UDP socket.
    memset(&hints, 0, sizeof(hints));
    hints.ai_family = AF_UNSPEC;
    hints.ai_socktype = SOCK_DGRAM;
    hints.ai_protocol = 0;
    hints.ai_flags = AI_PASSIVE|AI_ADDRCONFIG;

    err = getaddrinfo(HOST, PORT, &hints, &ai);
    if (err != 0) {
        fprintf(stderr, "failed to resolve local socket address (err=%d)",err);
        return 1;
    }

    // create an UDP socket.
    fd = socket(ai->ai_family, ai->ai_socktype, ai->ai_protocol);
    if (fd==-1) {
        fprintf(stderr, "%s", strerror(errno));
        return 1;
    }

    // bind the UDP socket.
    if (bind(fd, ai->ai_addr, ai->ai_addrlen) == -1) {
        fprintf(stderr, "%s", strerror(errno));
        return 1;
    }

    freeaddrinfo(ai);
    setup_handlers();

    // socket is set up; initialize the LEDs.
    ledstrip_init();
    ledstrip_clear();
    ledstrip_render();

    while (running) {
        received = recvfrom(fd, buffer, sizeof(buffer), 0, (struct sockaddr*)&src_addr, &src_addr_len);
        ++count;
        if (received == -1) {
            fprintf(stderr, "%s", strerror(errno));
        } else if (received == sizeof(buffer)) {
            fprintf(stderr, "datagram too large for buffer: truncated\n");
        } else {
            led = led__unpack(NULL, received, (const uint8_t *)buffer);
            if (led == NULL) {
                fprintf(stderr, "invalid protobuf message with size %d received; discarding.\n", received);
                continue;
            }
            ledstrip_assign(led->index, led->rgb);
            if (led->render) {
                ledstrip_render();
            }
            led__free_unpacked(led, NULL);
        }
    }

    ledstrip_clear();
    ledstrip_finish();
    close(fd);

    printf("\nProcessed %llu datagrams.\n", count-1);

    return 0;
}
