/*
 * LEDServer by Kim Tore Jensen <https://github.com/ambientsound/wirelight>.
 *
 * This is the server component. It listens on an UDP socket for Protobuf messages.
 */

#include <errno.h>
#include <string.h>
#include <unistd.h>
#include <netdb.h>
#include <sys/socket.h>
#include <netinet/in.h>

#include <stdio.h>
#include <stdarg.h>
#include <stdlib.h>

#define HOST "0.0.0.0"
#define PORT "1230"

void die(const char * fmt, ...) {
    va_list valist;
    vprintf(fmt, valist);
    exit(1);
}

void serve() {
    int fd;
    int err;

    struct addrinfo hints;
    struct addrinfo* res=0;

    memset(&hints, 0, sizeof(hints));
    hints.ai_family = AF_UNSPEC;
    hints.ai_socktype = SOCK_DGRAM;
    hints.ai_protocol = 0;
    hints.ai_flags = AI_PASSIVE|AI_ADDRCONFIG;

    err = getaddrinfo(HOST, PORT, &hints, &res);
    if (err != 0) {
        die("failed to resolve local socket address (err=%d)",err);
    }

    fd = socket(res->ai_family, res->ai_socktype, res->ai_protocol);
    if (fd==-1) {
        die("%s",strerror(errno));
    }

    freeaddrinfo(res);

    if (bind(fd,res->ai_addr,res->ai_addrlen)==-1) {
        die("%s", strerror(errno));
    }

    char buffer[549];
    struct sockaddr_storage src_addr;
    socklen_t src_addr_len=sizeof(src_addr);
    ssize_t count=recvfrom(fd,buffer,sizeof(buffer),0,(struct sockaddr*)&src_addr,&src_addr_len);
    if (count==-1) {
        die("%s",strerror(errno));
    } else if (count==sizeof(buffer)) {
        //warn("datagram too large for buffer: truncated");
    } else {
        //handle_datagram(buffer,count);
    }

    close(fd);
}
