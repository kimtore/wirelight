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
    int fd;
    int err;
    struct addrinfo hints;
    struct addrinfo* ai = 0;
    char buffer[548];
    struct sockaddr_storage src_addr;
    socklen_t src_addr_len = sizeof(src_addr);
    ssize_t received;

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

    uint32_t count = 0;
    while (running) {
        received = recvfrom(fd, buffer, sizeof(buffer), 0, (struct sockaddr*)&src_addr, &src_addr_len);
        if (received == -1) {
            fprintf(stderr, "%s", strerror(errno));
        } else if (received == sizeof(buffer)) {
            printf("datagram too large for buffer: truncated\n");
        } else {
            printf("received datagram of size %d\n", received);
            ledstrip_assign(count, 255*(count+1));
            ledstrip_render();
            ++count;
        }
    }

    ledstrip_clear();
    ledstrip_finish();
    close(fd);

    printf("\n");

    return 0;
}
