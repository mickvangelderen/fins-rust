#include <errno.h>
#include <netinet/in.h>
#include <signal.h>
#include <stdio.h>
#include <sys/socket.h>
#include <sys/types.h>

#define FINS_TCP_PORT 9600
#define SERV_IP_ADDR "196.36.32.100"
#define MAX_MSG 2010
#define MAX_HEADER 32
#define RESP_TIMEOUT 2
/*
 * FINS/TCP
 */
main(argc, argv) int argc;
char *argv[];
{
  int sockfd;
  struct sockaddr_in ws_addr, cs_addr;
  unsigned char fins_cmnd[MAX_MSG], fins_resp[MAX_MSG],
      fins_tcp_header[MAX_HEADER];
  unsigned char srv_node_no, cli_node_no;
  int sendlen, recvlen;
  char sid = 0;
  extern recv_fail();

  if ((sockfd = socket(AF_INET, SOCK_STREAM, 0)) < 0)
    err_exit("can't open stream socket");

  bzero((char *)&ws_addr, sizeof(ws_addr));
  ws_addr.sin_family = AF_INET;
  ws_addr.sin_addr.s_addr = htonl(INADDR_ANY);
  ws_addr.sin_port = htons(0); /*

  if (bind(sockfd,(struct sockaddr *)&ws_addr,sizeof(ws_addr)) < 0)
  err_exit(“can’t bind local address”);

  /* ESTABLISH CONNECTION WITH FINS/TCP SERVER*/
  bzero((char *)&cs_addr, sizeof(cs_addr));
  cs_addr.sin_family = AF_INET;
  cs_addr.sin_addr.s_addr = inet_addr(SERV_IP_ADDR);
  cs_addr.sin_port = htons(FINS_TCP_PORT);

  if (connect(sockfd, (struct sockaddr *)&cs_addr, sizeof(cs_addr)) < 0)
    err_exit(“can’t connect to FINS / TCP server”);

  /* SEND FINS/TCP COMMAND*/
  /*
   * GENERATE FINS NODE NUMBER DATA SEND COMMAND (CLIENT TO SERVER)
   */
  fins_tcp_header[0] = ‘F’; /* Header */
  fins_tcp_header[1] = ‘I’;
  fins_tcp_header[2] = ‘N’;
  fins_tcp_header[3] = ‘S’;
  fins_tcp_header[4] = 0x00; /* Length */
  fins_tcp_header[5] = 0x00;
  fins_tcp_header[6] = 0x00;
  fins_tcp_header[7] = 0x0C;
  fins_tcp_header[8] = 0x00; /* Command */
  fins_tcp_header[9] = 0x00;
  fins_tcp_header[10] = 0x00;
  fins_tcp_header[11] = 0x00;
  fins_tcp_header[12] = 0x00; /* Error Code */
  fins_tcp_header[13] = 0x00;
  fins_tcp_header[14] = 0x00;
  fins_tcp_header[15] = 0x00;
  fins_tcp_header[17] = 0x00; /* Client Node Add */
  fins_tcp_header[18] = 0x00;
  fins_tcp_header[19] = 0x00;
  fins_tcp_header[20] = 0x00;

  sendlen = 20;

  if (send(sockfd, fins_tcp_header, sendlen, 0) == sendlen) {
    alarm(RESP_TIMEOUT);
    printf(“FINS / TCP header send length % d¥n”, sendlen);
  }

  else {
    err_exit("FINS/TCP header send error");
  }

  /*RECEIVE FINS/TCP COMMAND (READ RECEIVE FUNCTIONS)*/
  recvlen = 24;
  if (tcp_recv(sockfd, fins_tcp_header, recvlen) == recvlen) {
    alarm(0); /*STOP RESPONSE MONITOR TIMER*/

    /* CONFIRM WHETHER FINS NODE NUMBER SEND COMMAND
     (CLIENT TO SERVER) WAS RECEIVED*/
    if ((fins_tcp_header[8] != 0x00) || (fins_tcp_header[9] != 0x00) ||
        (fins_tcp_header[10] != 0x00) || (fins_tcp_header[11] != 0x01)) {
      err_exit("FINS/TCP illegal commmand error");
    }

    printf("FINS/TCP header receive length %d¥n", recvlen);
    cli_node_no = fins_tcp_header[19];
    srv_node_no = fins_tcp_header[23];
    printf("FINS/TCP client Node No. = %d¥n", cli_node_no);
    printf("FINS/TCP server Node No. = %d¥n", srv_node_no);
  } else {
    err_exit("TCP receive error");
  }

  /* SEND FINS/TCP COMMAND*/
  /*
   * GENERATE FINS COMMAND FRAME
   */
  fins_tcp_header[0] = 'F'; /* Header */
  fins_tcp_header[1] = 'I';
  fins_tcp_header[2] = 'N';
  fins_tcp_header[3] = 'S';
  fins_tcp_header[4] = 0x00; /* Length */
  fins_tcp_header[5] = 0x00;
  fins_tcp_header[6] = 0x00;
  fins_tcp_header[7] =
      8 + 18; /*Length of data from Command up to end of FINS frame */
  fins_tcp_header[8] = 0x00; /* Command */
  fins_tcp_header[9] = 0x00;
  fins_tcp_header[10] = 0x00;
  fins_tcp_header[11] = 0x02;
  fins_tcp_header[12] = 0x00; /* Error Code */
  fins_tcp_header[13] = 0x00;
  fins_tcp_header[14] = 0x00;
  fins_tcp_header[15] = 0x00;

  /* SEND FINS/TCP COMMAND*/
  sendlen = 16;
  if (send(sockfd, fins_tcp_header, sendlen, 0) == sendlen) {

    alarm(RESP_TIMEOUT);
    printf("FINS/TCP header send length %d¥n", sendlen);
  } else {
    err_exit("FINS/TCP header send error");
  }

  /* SEND FINS COMMAND FRAME*/
  /*
   *
   * (READ 150 WORDS FROM DM 100)
   */
  fins_cmnd[0] = 0x80;        /* ICF */
  fins_cmnd[1] = 0x00;        /* RSV */
  fins_cmnd[2] = 0x02;        /* GCT */
  fins_cmnd[3] = 0x00;        /* DNA */
  fins_cmnd[4] = srv_node_no; /* DA1 */
  fins_cmnd[5] = 0x00;        /* DA2 */
  fins_cmnd[6] = 0x00;        /* SNA */
  fins_cmnd[7] = cli_node_no; /* SA1 */

  fins_cmnd[8] = 0x00;  /* SA2 */
  fins_cmnd[9] = ++sid; /* SID */
  fins_cmnd[10] = 0x01; /* MRC */
  fins_cmnd[11] = 0x01; /* SRC */
  fins_cmnd[12] = 0x82;
  fins_cmnd[13] = 0x00;
  fins_cmnd[14] = 0x64;
  fins_cmnd[15] = 0x00;
  fins_cmnd[16] = 0x00;
  fins_cmnd[17] = 0x96;
  /* SEND FINS COMMAND FRAME*/
  signal(SIGALRM, recv_fail);
  sendlen = 18;
  if (send(sockfd, fins_cmnd, sendlen, 0) == sendlen) {
    printf("send length %d¥n", sendlen);
  } else {
    err_exit("send error");
  }

  /* RECEIVE FINS/TCP COMMAND (READ RECEIVE FUNCTIONS)*/
  recvlen = 16;
  if (tcp_recv(sockfd, fins_tcp_header, recvlen) == recvlen) {
    /* CONFIRM WHETHER FINS FRAME SEND COMMAND WAS RECEIVED*/
    if ((fins_tcp_header[8] != 0x00) || (fins_tcp_header[9] != 0x00) ||
        (fins_tcp_header[10] != 0x00) || (fins_tcp_header[11] != 0x02)) {
      err_exit("FINS/TCP illegal commmand error");
    }

    printf("FINS/TCP header receive length %d¥n", recvlen);
    recvlen = fins_tcp_header[6];
    recvlen <<= 8;
    recvlen += fins_tcp_header[7];
    recvlen -=
        8; /* SUBTRACT LENGTH OF COMMAND & ERROR CODE OF FINS/TCP HEADER*/
    printf("FINS/TCP frame receive length %d¥n", recvlen);
  } else {
    err_exit("TCP receive error");
  }

  /* RECEIVE FINS RESPONSE FRAME*/
  if (tcp_recv(sockfd, fins_resp, recvlen) == recvlen) {
    alarm(0);
    printf("recv length %d¥n", recvlen);

    if (recvlen < 14)
      err_exit("FINS length error");
    if ((fins_cmnd[3] != fins_resp[6]) || (fins_cmnd[4] != fins_resp[7]) ||
        (fins_cmnd[5] != fins_resp[8])) {
      err_exit("illegal source address error");
    }

    if (fins_cmnd[9] != fins_resp[9]) /* SID CHECK */
      err_exit("illegal SID error");
  } else {
    alarm(0);
    err_exit("receive error");
  }

  /* */
  close(sockfd);
}

/*
 * TCP
 */
int tcp_recv(sockfd, buf, len) int sockfd;
unsigned char *buf;
int len;
{
  int total_len = 0;
  int recv_len;

  for (;;) {
    recv_len = recv(sockfd, (char *)buf, len, 0);

    if (recv_len > 0) {
      if (recv_len < (int)len) {
        len -= recv_len;
        buf += recv_len;
        total_len += recv_len;
      } else {
        total_len += recv_len;
        break;
      }
    } else {
      err_exit("TCP receive error");
      total_len = 0;
      break;
    }
  }

  return total_len;
}

/*
 *
 */
err_exit(err_msg) char *err_msg;
{
  printf("client: %s %x¥n", err_msg, errno);
  exit(1);
}

/*
 *
 */
recv_fail() { printf("response timeout error ¥n"); }