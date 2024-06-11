/*
 * Copyright (C) 2015-2023 IoT.bzh Company
 * Author: Fulup Ar Foll <fulup@iot.bzh>
 *
 * $RP_BEGIN_LICENSE$
 * Commercial License Usage
 *  Licensees holding valid commercial IoT.bzh licenses may use this file in
 *  accordance with the commercial license agreement provided with the
 *  Software or, alternatively, in accordance with the terms contained in
 *  a written agreement between you and The IoT.bzh Company. For licensing terms
 *  and conditions see https://www.iot.bzh/terms-conditions. For further
 *  information use the contact form at https://www.iot.bzh/contact.
 *
 * GNU General Public License Usage
 *  Alternatively, this file may be used under the terms of the GNU General
 *  Public license version 3. This license is as published by the Free Software
 *  Foundation and appearing in the file LICENSE.GPLv3 included in the packaging
 *  of this file. Please review the following information to ensure the GNU
 *  General Public License requirements will be met
 *  https://www.gnu.org/licenses/gpl-3.0.html.
 * $RP_END_LICENSE$
 */

#define AFB_BINDING_VERSION 4
// process with gcc -E libafb_glue.h | sed 's/static *inline//' | sed '/^#/d' | sed '/^$/d' >libafb_glue.c
#include <afb/afb-binding-v4.h>
#include <sys/epoll.h>
#include <errno.h>
#include <string.h>


//void afb_verbose(int loglevel, const char *file, int line, const char *function, const char *fmt, ...) __attribute__((format(printf, 5, 6)));

void afb_verbose(
int level,
const char *file,
int line,
const char *func,
const char *fmt,
...
) {
va_list args;
va_start (args, fmt);
afb_api_vverbose(afbBindingV4root, level, file, line, func, fmt, args);
va_end(args);
}

typedef enum afb_epoll {
  epoll_IN  = EPOLLIN,
  epoll_OUT = EPOLLOUT,
  epoll_HUP = EPOLLHUP,
  epoll_RDH = EPOLLRDHUP,
  epoll_ERR = EPOLLERR,
} afb_epoll_t;
