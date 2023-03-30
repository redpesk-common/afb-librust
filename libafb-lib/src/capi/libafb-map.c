       
       
       
enum afb_syslog_levels {
 AFB_SYSLOG_LEVEL_EMERGENCY = 0,
 AFB_SYSLOG_LEVEL_ALERT = 1,
 AFB_SYSLOG_LEVEL_CRITICAL = 2,
 AFB_SYSLOG_LEVEL_ERROR = 3,
 AFB_SYSLOG_LEVEL_WARNING = 4,
 AFB_SYSLOG_LEVEL_NOTICE = 5,
 AFB_SYSLOG_LEVEL_INFO = 6,
 AFB_SYSLOG_LEVEL_DEBUG = 7
};
       
enum afb_auth_type
{
 afb_auth_No = 0,
 afb_auth_Token,
 afb_auth_LOA,
 afb_auth_Permission,
 afb_auth_Or,
 afb_auth_And,
 afb_auth_Not,
 afb_auth_Yes
};
typedef enum afb_auth_type afb_auth_type_t;
struct afb_auth
{
 enum afb_auth_type type;
 union {
  const char *text;
  unsigned loa;
  const struct afb_auth *first;
 };
 const struct afb_auth *next;
};
typedef struct afb_auth afb_auth_t;
       
enum afb_ctlid
{
 afb_ctlid_Root_Entry = 0,
 afb_ctlid_Pre_Init = 1,
 afb_ctlid_Init = 2,
 afb_ctlid_Class_Ready = 3,
 afb_ctlid_Orphan_Event = 4,
 afb_ctlid_Exiting = 5
};
union afb_ctlarg
{
 struct {
  const char *path;
  const char *uid;
  struct json_object *config;
 } root_entry;
 struct {
  const char *path;
  const char *uid;
  struct json_object *config;
 } pre_init;
 struct {
  const char *name;
 } orphan_event;
 struct {
  int code;
 } exiting;
};
typedef enum afb_ctlid afb_ctlid_t;
typedef const union afb_ctlarg *afb_ctlarg_t;
       
enum afb_session_flags
{
       AFB_SESSION_LOA_MASK = 3,
       AFB_SESSION_LOA_0 = 0,
       AFB_SESSION_LOA_1 = 1,
       AFB_SESSION_LOA_2 = 2,
       AFB_SESSION_LOA_3 = 3,
       AFB_SESSION_CHECK = 4,
       AFB_SESSION_CLOSE = 16,
       AFB_SESSION_NONE = 0
};
       
enum afb_req_subcall_flags
{
 afb_req_subcall_catch_events = 1,
 afb_req_subcall_pass_events = 2,
 afb_req_subcall_on_behalf = 4,
 afb_req_subcall_api_session = 8,
};
typedef enum afb_req_subcall_flags afb_req_subcall_flags_t;
       
typedef unsigned char __u_char;
typedef unsigned short int __u_short;
typedef unsigned int __u_int;
typedef unsigned long int __u_long;
typedef signed char __int8_t;
typedef unsigned char __uint8_t;
typedef signed short int __int16_t;
typedef unsigned short int __uint16_t;
typedef signed int __int32_t;
typedef unsigned int __uint32_t;
typedef signed long int __int64_t;
typedef unsigned long int __uint64_t;
typedef __int8_t __int_least8_t;
typedef __uint8_t __uint_least8_t;
typedef __int16_t __int_least16_t;
typedef __uint16_t __uint_least16_t;
typedef __int32_t __int_least32_t;
typedef __uint32_t __uint_least32_t;
typedef __int64_t __int_least64_t;
typedef __uint64_t __uint_least64_t;
typedef long int __quad_t;
typedef unsigned long int __u_quad_t;
typedef long int __intmax_t;
typedef unsigned long int __uintmax_t;
typedef unsigned long int __dev_t;
typedef unsigned int __uid_t;
typedef unsigned int __gid_t;
typedef unsigned long int __ino_t;
typedef unsigned long int __ino64_t;
typedef unsigned int __mode_t;
typedef unsigned long int __nlink_t;
typedef long int __off_t;
typedef long int __off64_t;
typedef int __pid_t;
typedef struct { int __val[2]; } __fsid_t;
typedef long int __clock_t;
typedef unsigned long int __rlim_t;
typedef unsigned long int __rlim64_t;
typedef unsigned int __id_t;
typedef long int __time_t;
typedef unsigned int __useconds_t;
typedef long int __suseconds_t;
typedef int __daddr_t;
typedef int __key_t;
typedef int __clockid_t;
typedef void * __timer_t;
typedef long int __blksize_t;
typedef long int __blkcnt_t;
typedef long int __blkcnt64_t;
typedef unsigned long int __fsblkcnt_t;
typedef unsigned long int __fsblkcnt64_t;
typedef unsigned long int __fsfilcnt_t;
typedef unsigned long int __fsfilcnt64_t;
typedef long int __fsword_t;
typedef long int __ssize_t;
typedef long int __syscall_slong_t;
typedef unsigned long int __syscall_ulong_t;
typedef __off64_t __loff_t;
typedef char *__caddr_t;
typedef long int __intptr_t;
typedef unsigned int __socklen_t;
typedef int __sig_atomic_t;
typedef __int8_t int8_t;
typedef __int16_t int16_t;
typedef __int32_t int32_t;
typedef __int64_t int64_t;
typedef __uint8_t uint8_t;
typedef __uint16_t uint16_t;
typedef __uint32_t uint32_t;
typedef __uint64_t uint64_t;
typedef __int_least8_t int_least8_t;
typedef __int_least16_t int_least16_t;
typedef __int_least32_t int_least32_t;
typedef __int_least64_t int_least64_t;
typedef __uint_least8_t uint_least8_t;
typedef __uint_least16_t uint_least16_t;
typedef __uint_least32_t uint_least32_t;
typedef __uint_least64_t uint_least64_t;
typedef signed char int_fast8_t;
typedef long int int_fast16_t;
typedef long int int_fast32_t;
typedef long int int_fast64_t;
typedef unsigned char uint_fast8_t;
typedef unsigned long int uint_fast16_t;
typedef unsigned long int uint_fast32_t;
typedef unsigned long int uint_fast64_t;
typedef long int intptr_t;
typedef unsigned long int uintptr_t;
typedef __intmax_t intmax_t;
typedef __uintmax_t uintmax_t;
typedef long int ptrdiff_t;
typedef long unsigned int size_t;
typedef int wchar_t;
typedef struct {
  long long __max_align_ll __attribute__((__aligned__(__alignof__(long long))));
  long double __max_align_ld __attribute__((__aligned__(__alignof__(long double))));
} max_align_t;
typedef __builtin_va_list __gnuc_va_list;
typedef __gnuc_va_list va_list;
typedef __clock_t clock_t;
typedef __time_t time_t;
struct tm
{
  int tm_sec;
  int tm_min;
  int tm_hour;
  int tm_mday;
  int tm_mon;
  int tm_year;
  int tm_wday;
  int tm_yday;
  int tm_isdst;
  long int tm_gmtoff;
  const char *tm_zone;
};
struct timespec
{
  __time_t tv_sec;
  __syscall_slong_t tv_nsec;
};
typedef __clockid_t clockid_t;
typedef __timer_t timer_t;
struct itimerspec
  {
    struct timespec it_interval;
    struct timespec it_value;
  };
struct sigevent;
typedef __pid_t pid_t;
struct __locale_struct
{
  struct __locale_data *__locales[13];
  const unsigned short int *__ctype_b;
  const int *__ctype_tolower;
  const int *__ctype_toupper;
  const char *__names[13];
};
typedef struct __locale_struct *__locale_t;
typedef __locale_t locale_t;
extern clock_t clock (void) __attribute__ ((__nothrow__ , __leaf__));
extern time_t time (time_t *__timer) __attribute__ ((__nothrow__ , __leaf__));
extern double difftime (time_t __time1, time_t __time0)
     __attribute__ ((__nothrow__ , __leaf__)) __attribute__ ((__const__));
extern time_t mktime (struct tm *__tp) __attribute__ ((__nothrow__ , __leaf__));
extern size_t strftime (char *__restrict __s, size_t __maxsize,
   const char *__restrict __format,
   const struct tm *__restrict __tp) __attribute__ ((__nothrow__ , __leaf__));
extern size_t strftime_l (char *__restrict __s, size_t __maxsize,
     const char *__restrict __format,
     const struct tm *__restrict __tp,
     locale_t __loc) __attribute__ ((__nothrow__ , __leaf__));
extern struct tm *gmtime (const time_t *__timer) __attribute__ ((__nothrow__ , __leaf__));
extern struct tm *localtime (const time_t *__timer) __attribute__ ((__nothrow__ , __leaf__));
extern struct tm *gmtime_r (const time_t *__restrict __timer,
       struct tm *__restrict __tp) __attribute__ ((__nothrow__ , __leaf__));
extern struct tm *localtime_r (const time_t *__restrict __timer,
          struct tm *__restrict __tp) __attribute__ ((__nothrow__ , __leaf__));
extern char *asctime (const struct tm *__tp) __attribute__ ((__nothrow__ , __leaf__));
extern char *ctime (const time_t *__timer) __attribute__ ((__nothrow__ , __leaf__));
extern char *asctime_r (const struct tm *__restrict __tp,
   char *__restrict __buf) __attribute__ ((__nothrow__ , __leaf__));
extern char *ctime_r (const time_t *__restrict __timer,
        char *__restrict __buf) __attribute__ ((__nothrow__ , __leaf__));
extern char *__tzname[2];
extern int __daylight;
extern long int __timezone;
extern char *tzname[2];
extern void tzset (void) __attribute__ ((__nothrow__ , __leaf__));
extern int daylight;
extern long int timezone;
extern time_t timegm (struct tm *__tp) __attribute__ ((__nothrow__ , __leaf__));
extern time_t timelocal (struct tm *__tp) __attribute__ ((__nothrow__ , __leaf__));
extern int dysize (int __year) __attribute__ ((__nothrow__ , __leaf__)) __attribute__ ((__const__));
extern int nanosleep (const struct timespec *__requested_time,
        struct timespec *__remaining);
extern int clock_getres (clockid_t __clock_id, struct timespec *__res) __attribute__ ((__nothrow__ , __leaf__));
extern int clock_gettime (clockid_t __clock_id, struct timespec *__tp) __attribute__ ((__nothrow__ , __leaf__));
extern int clock_settime (clockid_t __clock_id, const struct timespec *__tp)
     __attribute__ ((__nothrow__ , __leaf__));
extern int clock_nanosleep (clockid_t __clock_id, int __flags,
       const struct timespec *__req,
       struct timespec *__rem);
extern int clock_getcpuclockid (pid_t __pid, clockid_t *__clock_id) __attribute__ ((__nothrow__ , __leaf__));
extern int timer_create (clockid_t __clock_id,
    struct sigevent *__restrict __evp,
    timer_t *__restrict __timerid) __attribute__ ((__nothrow__ , __leaf__));
extern int timer_delete (timer_t __timerid) __attribute__ ((__nothrow__ , __leaf__));
extern int timer_settime (timer_t __timerid, int __flags,
     const struct itimerspec *__restrict __value,
     struct itimerspec *__restrict __ovalue) __attribute__ ((__nothrow__ , __leaf__));
extern int timer_gettime (timer_t __timerid, struct itimerspec *__value)
     __attribute__ ((__nothrow__ , __leaf__));
extern int timer_getoverrun (timer_t __timerid) __attribute__ ((__nothrow__ , __leaf__));
extern int timespec_get (struct timespec *__ts, int __base)
     __attribute__ ((__nothrow__ , __leaf__)) __attribute__ ((__nonnull__ (1)));
struct afb_verb_v4;
struct afb_api_x4;
struct afb_req_x4;
struct afb_event_x4;
struct afb_type_x4;
struct afb_data_x4;
struct afb_evfd_x4;
struct afb_timer_x4;
typedef struct afb_api_x4 *afb_api_x4_t;
typedef struct afb_req_x4 *afb_req_x4_t;
typedef struct afb_event_x4 *afb_event_x4_t;
typedef struct afb_data_x4 *afb_data_x4_t;
typedef struct afb_type_x4 *afb_type_x4_t;
typedef struct afb_evfd_x4 *afb_evfd_x4_t;
typedef struct afb_timer_x4 *afb_timer_x4_t;
typedef
 enum afb_type_flags_x4
{
 Afb_Type_Flags_x4_Shareable = 1,
 Afb_Type_Flags_x4_Streamable = 2,
 Afb_Type_Flags_x4_Opaque = 4
}
 afb_type_flags_x4_t;
typedef int (*afb_api_callback_x4_t)(
  afb_api_x4_t api,
  afb_ctlid_t ctlid,
  afb_ctlarg_t ctlarg,
  void *userdata);
typedef void (*afb_req_callback_x4_t)(
  afb_req_x4_t req,
  unsigned nparams,
  afb_data_x4_t const params[]);
typedef void (*afb_call_callback_x4_t)(
  void *closure,
  int status,
  unsigned nreplies,
  afb_data_x4_t const replies[],
  afb_api_x4_t api);
typedef void (*afb_subcall_callback_x4_t)(
  void *closure,
  int status,
  unsigned nreplies,
  afb_data_x4_t const replies[],
  afb_req_x4_t req);
typedef void (*afb_check_callback_x4_t)(
  void *closure,
  int status,
  afb_req_x4_t req);
typedef void (*afb_event_handler_x4_t)(
  void *closure,
  const char *event_name,
  unsigned nparams,
  afb_data_x4_t const params[],
  afb_api_x4_t api);
typedef int (*afb_type_converter_x4_t)(
  void *closure,
  afb_data_x4_t from,
  afb_type_x4_t type,
  afb_data_x4_t *to);
typedef int (*afb_type_updater_x4_t)(
  void *closure,
  afb_data_x4_t from,
  afb_type_x4_t type,
  afb_data_x4_t to);
typedef void (*afb_evfd_handler_x4_t)(afb_evfd_x4_t efd, int fd, uint32_t revents, void *closure);
typedef void (*afb_timer_handler_x4_t)(afb_timer_x4_t timer, void *closure, unsigned decount);
struct afb_binding_x4r1_itf
{
 int (*create_data_raw)(
  afb_data_x4_t *data,
  afb_type_x4_t type,
  const void *buffer,
  size_t size,
  void (*dispose)(void*),
  void *closure);
 int (*create_data_alloc)(
  afb_data_x4_t *data,
  afb_type_x4_t type,
  void **pointer,
  size_t size);
 int (*create_data_copy)(
  afb_data_x4_t *data,
  afb_type_x4_t type,
  const void *pointer,
  size_t size);
 afb_data_x4_t(*data_addref)(
  afb_data_x4_t data);
 void (*data_unref)(
  afb_data_x4_t data);
 int (*data_get_mutable)(
  afb_data_x4_t data,
  void **pointer,
  size_t *size);
 int (*data_get_constant)(
  afb_data_x4_t data,
  void **pointer,
  size_t *size);
 int (*data_update)(
  afb_data_x4_t data,
  afb_data_x4_t value);
 int (*data_convert)(
  afb_data_x4_t data,
  afb_type_x4_t type,
  afb_data_x4_t *result);
 afb_type_x4_t (*data_type)(
  afb_data_x4_t data);
 void (*data_notify_changed)(
  afb_data_x4_t data);
 int (*data_is_volatile)(
  afb_data_x4_t data);
 void (*data_set_volatile)(
  afb_data_x4_t data);
 void (*data_set_not_volatile)(
  afb_data_x4_t data);
 int (*data_is_constant)(
  afb_data_x4_t data);
 void (*data_set_constant)(
  afb_data_x4_t data);
 void (*data_set_not_constant)(
  afb_data_x4_t data);
 void (*data_lock_read)(
  afb_data_x4_t data);
 int (*data_try_lock_read)(
  afb_data_x4_t data);
 void (*data_lock_write)(
  afb_data_x4_t data);
 int (*data_try_lock_write)(
  afb_data_x4_t data);
 void (*data_unlock)(
  afb_data_x4_t data);
 int (*req_logmask)(
  afb_req_x4_t req);
 afb_req_x4_t (*req_addref)(
  afb_req_x4_t req);
 void (*req_unref)(
  afb_req_x4_t req);
 afb_api_x4_t (*req_api)(
  afb_req_x4_t req);
 void *(*req_vcbdata)(
  afb_req_x4_t req);
 const char *(*req_called_api)(
  afb_req_x4_t req);
 const char *(*req_called_verb)(
  afb_req_x4_t req);
 void (*req_vverbose)(
  afb_req_x4_t req,
  int level,
  const char *file,
  int line,
  const char * func,
  const char *fmt,
  va_list args);
 void (*req_session_close)(
  afb_req_x4_t req);
 int (*req_session_set_LOA)(
  afb_req_x4_t req,
  unsigned level);
 void *(*LEGACY_req_cookie)(
  afb_req_x4_t req,
  int replace,
  void *(*createcb)(void *closure),
  void (*freecb)(void *value),
  void *closure);
 int (*req_subscribe)(
  afb_req_x4_t req,
  afb_event_x4_t event);
 int (*req_unsubscribe)(
  afb_req_x4_t req,
  afb_event_x4_t event);
 struct json_object *(*req_get_client_info)(
  afb_req_x4_t req);
 void (*req_check_permission)(
  afb_req_x4_t req,
  const char *permission,
  afb_check_callback_x4_t callback,
  void *closure);
 unsigned (*req_parameters)(
  afb_req_x4_t req,
  afb_data_x4_t const **params);
 void (*req_reply)(
  afb_req_x4_t req,
  int status,
  unsigned nreplies,
  afb_data_x4_t const *replies);
 void (*req_subcall)(
  afb_req_x4_t req,
  const char *apiname,
  const char *verbname,
  unsigned nparams,
  afb_data_x4_t const params[],
  int flags,
  afb_subcall_callback_x4_t callback,
  void *closure);
 int (*req_subcall_sync)(
  afb_req_x4_t req,
  const char *apiname,
  const char *verbname,
  unsigned nparams,
  afb_data_x4_t const params[],
  int flags,
  int *status,
  unsigned *nreplies,
  afb_data_x4_t replies[]);
 afb_event_x4_t (*event_addref)(
  afb_event_x4_t event);
 void (*event_unref)(
  afb_event_x4_t event);
 const char *(*event_name)(
  afb_event_x4_t event);
 int (*event_push)(
  afb_event_x4_t event,
  unsigned nparams,
  afb_data_x4_t const params[]);
 int (*event_broadcast)(
  afb_event_x4_t event,
  unsigned nparams,
  afb_data_x4_t const params[]);
 int (*type_lookup)(
  afb_type_x4_t *type,
  const char *name);
 int (*type_register)(
  afb_type_x4_t *type,
  const char *name,
  afb_type_flags_x4_t flags);
 const char *(*type_name)(
  afb_type_x4_t type
  );
 int (*type_set_family)(
  afb_type_x4_t type,
  afb_type_x4_t family
  );
 int (*type_add_converter)(
  afb_type_x4_t fromtype,
  afb_type_x4_t totype,
  afb_type_converter_x4_t converter,
  void *closure
  );
 int (*type_add_updater)(
  afb_type_x4_t fromtype,
  afb_type_x4_t totype,
  afb_type_updater_x4_t updater,
  void *closure
  );
 int (*api_logmask)(
  afb_api_x4_t api);
 void (*api_vverbose)(
  afb_api_x4_t api,
  int level,
  const char *file,
  int line,
  const char * func,
  const char *fmt,
  va_list args);
 const char *(*api_name)(
  afb_api_x4_t api);
 void *(*api_get_userdata)(
  afb_api_x4_t api);
 void *(*api_set_userdata)(
  afb_api_x4_t api,
  void *userdata);
 struct json_object *(*api_settings)(
  afb_api_x4_t api);
 int (*api_event_broadcast)(
  afb_api_x4_t api,
  const char *name,
  unsigned nparams,
  afb_data_x4_t const params[]);
 int (*api_new_event)(
  afb_api_x4_t api,
  const char *name,
  afb_event_x4_t *event);
 int (*api_event_handler_add)(
  afb_api_x4_t api,
  const char *pattern,
  afb_event_handler_x4_t callback,
  void *closure);
 int (*api_event_handler_del)(
  afb_api_x4_t api,
  const char *pattern,
  void **closure);
 void (*api_call)(
  afb_api_x4_t api,
  const char *apiname,
  const char *verbname,
  unsigned nparams,
  afb_data_x4_t const params[],
  afb_call_callback_x4_t callback,
  void *closure);
 int (*api_call_sync)(
  afb_api_x4_t api,
  const char *apiname,
  const char *verbname,
  unsigned nparams,
  afb_data_x4_t const params[],
  int *status,
  unsigned *nreplies,
  afb_data_x4_t replies[]);
 int (*api_add_verb)(
  afb_api_x4_t api,
  const char *verb,
  const char *info,
  afb_req_callback_x4_t callback,
  void *vcbdata,
  const struct afb_auth *auth,
  uint32_t session,
  int glob);
 int (*api_del_verb)(
  afb_api_x4_t api,
  const char *verb,
  void **vcbdata);
 void (*api_seal)(
  afb_api_x4_t api);
 int (*api_set_verbs)(
  afb_api_x4_t api,
  const struct afb_verb_v4 *verbs);
 int (*api_require_api)(
  afb_api_x4_t api,
  const char *name,
  int initialized);
 int (*api_class_provide)(
  afb_api_x4_t api,
  const char *name);
 int (*api_class_require)(
  afb_api_x4_t api,
  const char *name);
 int (*api_delete)(
  afb_api_x4_t api);
 int (*create_api)(
  afb_api_x4_t root,
  afb_api_x4_t *newapi,
  const char *apiname,
  const char *info,
  int noconcurrency,
  afb_api_callback_x4_t mainctl,
  void *userdata);
 int (*job_post)(
  afb_api_x4_t root,
  long delayms,
  int timeout,
  void (*callback)(int signum, void *arg),
  void *argument,
  void *group);
 int (*alias_api)(
  afb_api_x4_t root,
  const char *name,
  const char *as_name);
 int (*setup_shared_object)(
  afb_api_x4_t root,
  void *handle);
 afb_type_x4_t type_opaque;
 afb_type_x4_t type_stringz;
 afb_type_x4_t type_json;
 afb_type_x4_t type_json_c;
 afb_type_x4_t type_bool;
 afb_type_x4_t type_i32;
 afb_type_x4_t type_u32;
 afb_type_x4_t type_i64;
 afb_type_x4_t type_u64;
 afb_type_x4_t type_double;
 int (*evfd_create)(
  afb_evfd_x4_t *efd,
  int fd,
  uint32_t events,
  afb_evfd_handler_x4_t handler,
  void *closure,
  int autounref,
  int autoclose);
 afb_evfd_x4_t (*evfd_addref)(
  afb_evfd_x4_t efd);
 void (*evfd_unref)(
  afb_evfd_x4_t efd);
 int (*evfd_get_fd)(
  afb_evfd_x4_t efd);
 uint32_t (*evfd_get_events)(
  afb_evfd_x4_t efd);
 void (*evfd_set_events)(
  afb_evfd_x4_t efd,
  uint32_t events);
 int (*timer_create)(
  afb_timer_x4_t *timer,
  int absolute,
  time_t start_sec,
  unsigned start_ms,
  unsigned count,
  unsigned period_ms,
  unsigned accuracy_ms,
  afb_timer_handler_x4_t handler,
  void *closure,
  int autounref);
 afb_timer_x4_t (*timer_addref)(
  afb_timer_x4_t timer);
 void (*timer_unref)(
  afb_timer_x4_t timer);
 unsigned (*req_session_get_LOA)(
  afb_req_x4_t req);
 int (*data_dependency_add)(
  afb_data_x4_t from_data,
  afb_data_x4_t to_data);
 int (*data_dependency_sub)(
  afb_data_x4_t from_data,
  afb_data_x4_t to_data);
 void (*data_dependency_drop_all)(
  afb_data_x4_t data);
 int (*req_cookie_set)(
  afb_req_x4_t req,
  void *value,
  void (*freecb)(void*),
  void *freeclo);
 int (*req_cookie_get)(
  afb_req_x4_t req,
  void **value);
 int (*req_cookie_getinit)(
  afb_req_x4_t req,
  void **value,
  int (*initcb)(void *closure, void **value, void (**freecb)(void*), void **freeclo),
  void *closure);
 int (*req_cookie_drop)(
  afb_req_x4_t req);
 afb_type_x4_t type_bytearray;
 int (*req_param_convert)(
  afb_req_x4_t req,
  unsigned index,
  afb_type_x4_t type,
  afb_data_x4_t *result);
 int (*req_interface_by_id)(
  afb_req_x4_t req,
  int id,
  void **result);
 int (*req_interface_by_name)(
  afb_req_x4_t req,
  const char *name,
  void **result);
 void *(*req_get_userdata)(
  afb_req_x4_t req);
 void (*req_set_userdata)(
  afb_req_x4_t req,
  void *userdata,
  void (*freecb)(void*));
 int (*job_abort)(
  afb_api_x4_t root,
  int jobid);
};
       
typedef afb_api_x4_t afb_api_t;
typedef afb_req_x4_t afb_req_t;
typedef afb_event_x4_t afb_event_t;
typedef afb_data_x4_t afb_data_t;
typedef afb_type_x4_t afb_type_t;
typedef afb_evfd_x4_t afb_evfd_t;
typedef afb_timer_x4_t afb_timer_t;
typedef afb_type_flags_x4_t afb_type_flags_t;
typedef afb_type_converter_x4_t afb_type_converter_t;
typedef afb_type_updater_x4_t afb_type_updater_t;
typedef afb_api_callback_x4_t afb_api_callback_t;
typedef afb_req_callback_x4_t afb_req_callback_t;
typedef afb_call_callback_x4_t afb_call_callback_t;
typedef afb_subcall_callback_x4_t afb_subcall_callback_t;
typedef afb_check_callback_x4_t afb_check_callback_t;
typedef afb_event_handler_x4_t afb_event_handler_t;
typedef afb_type_converter_x4_t afb_type_converter_t;
typedef afb_type_updater_x4_t afb_type_updater_t;
typedef afb_evfd_handler_x4_t afb_evfd_handler_t;
typedef afb_timer_handler_x4_t afb_timer_handler_t;
afb_api_t afbBindingV4root __attribute__((weak));
const struct afb_binding_x4r1_itf *afbBindingV4r1_itfptr __attribute__((weak));
short afbBindingV4_itf_revision __attribute__((weak)) = 5;
int
afb_data_is_valid(
 afb_data_t data
) {
 return data != 0;
}
int
afb_create_data_raw(
 afb_data_t *data,
 afb_type_t type,
 const void *buffer,
 size_t size,
 void (*dispose)(void*),
 void *closure
) {
 return afbBindingV4r1_itfptr->create_data_raw(data, type, buffer, size, dispose, closure);
}
int
afb_create_data_alloc(
 afb_data_t *data,
 afb_type_t type,
 void **pointer,
 size_t size
) {
 return afbBindingV4r1_itfptr->create_data_alloc(data, type, pointer, size);
}
int
afb_create_data_copy(
 afb_data_t *data,
 afb_type_t type,
 const void *buffer,
 size_t size
) {
 return afbBindingV4r1_itfptr->create_data_copy(data, type, buffer, size);
}
afb_data_t
afb_data_addref(
 afb_data_t data
) {
 return afbBindingV4r1_itfptr->data_addref(data);
}
void
afb_data_unref(
 afb_data_t data
) {
 afbBindingV4r1_itfptr->data_unref(data);
}
int
afb_data_convert(
 afb_data_t data,
 afb_type_t type,
 afb_data_t *result
) {
 return afbBindingV4r1_itfptr->data_convert(data, type, result);
}
afb_type_t
afb_data_type(
 afb_data_t data
) {
 return afbBindingV4r1_itfptr->data_type(data);
}
int
afb_data_get_mutable(
 afb_data_t data,
 void **pointer,
 size_t *size
) {
 return afbBindingV4r1_itfptr->data_get_mutable(data, pointer, size);
}
int
afb_data_get_constant(
 afb_data_t data,
 void **pointer,
 size_t *size
) {
 return afbBindingV4r1_itfptr->data_get_constant(data, pointer, size);
}
size_t
afb_data_size(
 afb_data_t data
) {
 size_t sz;
 afb_data_get_constant(data, 0, &sz);
 return sz;
}
void *
afb_data_ro_pointer(
 afb_data_t data
) {
 void *ptr;
 afb_data_get_constant(data, &ptr, 0);
 return ptr;
}
void *
afb_data_rw_pointer(
 afb_data_t data
) {
 void *ptr;
 afb_data_get_mutable(data, &ptr, 0);
 return ptr;
}
void
afb_data_notify_changed(
 afb_data_t data
) {
 afbBindingV4r1_itfptr->data_notify_changed(data);
}
int
afb_data_is_volatile(
 afb_data_t data
) {
 return afbBindingV4r1_itfptr->data_is_volatile(data);
}
void
afb_data_set_volatile(
 afb_data_t data
) {
 afbBindingV4r1_itfptr->data_set_volatile(data);
}
void
afb_data_set_not_volatile(
 afb_data_t data
) {
 afbBindingV4r1_itfptr->data_set_not_volatile(data);
}
int
afb_data_is_constant(
 afb_data_t data
) {
 return afbBindingV4r1_itfptr->data_is_constant(data);
}
void
afb_data_set_constant(
 afb_data_t data
) {
 afbBindingV4r1_itfptr->data_set_constant(data);
}
void
afb_data_set_not_constant(
 afb_data_t data
) {
 afbBindingV4r1_itfptr->data_set_not_constant(data);
}
void
afb_data_lock_read(
 afb_data_t data
) {
 afbBindingV4r1_itfptr->data_lock_read(data);
}
int
afb_data_try_lock_read(
 afb_data_t data
) {
 return afbBindingV4r1_itfptr->data_try_lock_read(data);
}
void
afb_data_lock_write(
 afb_data_t data
) {
 afbBindingV4r1_itfptr->data_lock_write(data);
}
int
afb_data_try_lock_write(
 afb_data_t data
) {
 return afbBindingV4r1_itfptr->data_try_lock_write(data);
}
void
afb_data_unlock(
 afb_data_t data
) {
 afbBindingV4r1_itfptr->data_unlock(data);
}
int
afb_data_update(
 afb_data_t data,
 afb_data_t value
) {
 return afbBindingV4r1_itfptr->data_update(data, value);
}
void
afb_data_assign(
 afb_data_t *data,
 afb_data_t value
) {
 afb_data_unref(*data);
 *data = value;
}
int
afb_data_dependency_add(
 afb_data_t from_data,
 afb_data_t to_data
) {
 return afbBindingV4r1_itfptr->data_dependency_add(from_data, to_data);
}
int
afb_data_dependency_sub(
 afb_data_t from_data,
 afb_data_t to_data
) {
 return afbBindingV4r1_itfptr->data_dependency_sub(from_data, to_data);
}
void
afb_data_dependency_drop_all(
 afb_data_t data
) {
 return afbBindingV4r1_itfptr->data_dependency_drop_all(data);
}
void
afb_data_array_addref(
 unsigned count,
 afb_data_t const *array
) {
 while (count--)
  afb_data_addref(*array++);
}
void
afb_data_array_unref(
 unsigned count,
 afb_data_t const *array
) {
 while (count--)
  afb_data_unref(*array++);
}
int
afb_data_array_convert(
 unsigned count,
 afb_data_t const * array_data,
 afb_type_t const * array_type,
 afb_data_t *array_result
) {
 int rc = 0;
 unsigned index = 0;
 while (rc >= 0 && index < count) {
  rc = afb_data_convert(array_data[index], array_type[index], &array_result[index]);
  if (rc >= 0)
   index++;
  else {
   while (index)
    afb_data_unref(array_result[--index]);
   while (index < count)
    array_result[index++] = 0;
  }
 }
 return rc;
}
int
afb_req_logmask(
 afb_req_t req
) {
 return afbBindingV4r1_itfptr->req_logmask(req);
}
int
afb_req_wants_log_level(
 afb_req_t req,
 int level
) {
 return ((afb_req_logmask(req)) & (1 << (level)));
}
int
afb_req_is_valid(
 afb_req_t req
) {
 return !!req;
}
afb_api_t
afb_req_get_api(
 afb_req_t req
) {
 return afbBindingV4r1_itfptr->req_api(req);
}
void *
afb_req_get_vcbdata(
 afb_req_t req
) {
 return afbBindingV4r1_itfptr->req_vcbdata(req);
}
const char *
afb_req_get_called_api(
 afb_req_t req
) {
 return afbBindingV4r1_itfptr->req_called_api(req);
}
const char *
afb_req_get_called_verb(
 afb_req_t req
) {
 return afbBindingV4r1_itfptr->req_called_verb(req);
}
afb_req_t
afb_req_addref(
 afb_req_t req
) {
 return afbBindingV4r1_itfptr->req_addref(req);
}
void
afb_req_unref(
 afb_req_t req
) {
 afbBindingV4r1_itfptr->req_unref(req);
}
void
afb_req_vverbose(
 afb_req_t req,
 int level, const char *file,
 int line,
 const char * func,
 const char *fmt,
 va_list args
) {
 afbBindingV4r1_itfptr->req_vverbose(req, level, file, line, func, fmt, args);
}
__attribute__((format(printf, 6, 7)))
void
afb_req_verbose(
 afb_req_t req,
 int level, const char *file,
 int line,
 const char * func,
 const char *fmt,
 ...
) {
 va_list args;
 __builtin_va_start(args,fmt);
 afb_req_vverbose(req, level, file, line, func, fmt, args);
 __builtin_va_end(args);
}
int
afb_req_context(
 afb_req_t req,
 void **ptrval,
 int (*initcb)(void *closure, void **value, void (**freecb)(void*), void **freeclo),
 void *closure
) {
 return afbBindingV4r1_itfptr->req_cookie_getinit(req, ptrval, initcb, closure);
}
int
afb_req_context_get(
 afb_req_t req,
 void **ptrval
) {
 return afbBindingV4r1_itfptr->req_cookie_get(req, ptrval);
}
int
afb_req_context_set(
 afb_req_t req,
 void *value,
 void (*freecb)(void*),
 void *freeclo
) {
 return afbBindingV4r1_itfptr->req_cookie_set(req, value, freecb, freeclo);
}
int
afb_req_context_drop(
 afb_req_t req
) {
 return afbBindingV4r1_itfptr->req_cookie_drop(req);
}
void
afb_req_session_close(
 afb_req_t req
) {
 afbBindingV4r1_itfptr->req_session_close(req);
}
int
afb_req_session_set_LOA(
 afb_req_t req,
 unsigned level
) {
 return afbBindingV4r1_itfptr->req_session_set_LOA(req, level);
}
unsigned
afb_req_session_get_LOA(
 afb_req_t req
) {
 return afbBindingV4r1_itfptr->req_session_get_LOA(req);
}
int
afb_req_subscribe(
 afb_req_t req,
 afb_event_t event
) {
 return afbBindingV4r1_itfptr->req_subscribe(req, event);
}
int
afb_req_unsubscribe(
 afb_req_t req,
 afb_event_t event
) {
 return afbBindingV4r1_itfptr->req_unsubscribe(req, event);
}
void
afb_req_check_permission(
 afb_req_t req,
 const char *permission,
 afb_check_callback_t callback,
 void *closure
) {
 afbBindingV4r1_itfptr->req_check_permission(req, permission, callback, closure);
}
struct json_object *
afb_req_get_client_info(
 afb_req_t req
) {
 return afbBindingV4r1_itfptr->req_get_client_info(req);
}
unsigned
afb_req_parameters(
 afb_req_t req,
 afb_data_t const **params
) {
 return afbBindingV4r1_itfptr->req_parameters(req, params);
}
int
afb_req_param_convert(
 afb_req_t req,
 unsigned index,
 afb_type_t type,
 afb_data_t *result
) {
 return afbBindingV4r1_itfptr->req_param_convert(req, index, type, result);
}
void
afb_req_reply(
 afb_req_t req,
 int status,
 unsigned nreplies,
 afb_data_t const *replies
) {
 afbBindingV4r1_itfptr->req_reply(req, status, nreplies, replies);
}
void
afb_req_subcall(
 afb_req_t req,
 const char *apiname,
 const char *verbname,
 unsigned nparams,
 afb_data_t const params[],
 int flags,
 afb_subcall_callback_t callback,
 void *closure
) {
 afbBindingV4r1_itfptr->req_subcall(req, apiname, verbname, nparams, params, flags, callback, closure);
}
int
afb_req_subcall_sync(
 afb_req_t req,
 const char *apiname,
 const char *verbname,
 unsigned nparams,
 afb_data_t const params[],
 int flags,
 int *status,
 unsigned *nreplies,
 afb_data_t replies[]
) {
 return afbBindingV4r1_itfptr->req_subcall_sync(req, apiname, verbname, nparams, params, flags, status, nreplies, replies);
}
int
afb_req_get_interface_by_id(
 afb_req_t req,
 int itfid,
 void **result
) {
 return afbBindingV4r1_itfptr->req_interface_by_id(req, itfid, result);
}
int
afb_req_get_interface_by_name(
 afb_req_t req,
 const char *name,
 void **result
) {
 return afbBindingV4r1_itfptr->req_interface_by_name(req, name, result);
}
void *
afb_req_get_userdata(
 afb_req_t req
) {
 return afbBindingV4r1_itfptr->req_get_userdata(req);
}
void
afb_req_set_userdata(
 afb_req_t req,
 void *userdata,
 void (*freecb)(void*)
) {
 afbBindingV4r1_itfptr->req_set_userdata(req, userdata, freecb);
}
int
afb_event_is_valid(
 afb_event_t event
) {
 return !!event;
}
afb_event_t
afb_event_addref(
 afb_event_t event
) {
 return afbBindingV4r1_itfptr->event_addref(event);
}
void
afb_event_unref(
 afb_event_t event
) {
 afbBindingV4r1_itfptr->event_unref(event);
}
const char *
afb_event_name(
 afb_event_t event
) {
 return afbBindingV4r1_itfptr->event_name(event);
}
int
afb_event_push(
 afb_event_t event,
 unsigned nparams,
 afb_data_t const params[]
) {
 return afbBindingV4r1_itfptr->event_push(event, nparams, params);
}
int
afb_event_broadcast(
 afb_event_t event,
 unsigned nparams,
 afb_data_t const params[]
) {
 return afbBindingV4r1_itfptr->event_broadcast(event, nparams, params);
}
int
afb_type_lookup(
 afb_type_t *type,
 const char *name
) {
 return afbBindingV4r1_itfptr->type_lookup(type, name);
}
int
afb_type_register(
 afb_type_t *type,
 const char *name,
 afb_type_flags_t flags
) {
 return afbBindingV4r1_itfptr->type_register(type, name, flags);
}
const char *
afb_type_name(
 afb_type_t type
) {
 return afbBindingV4r1_itfptr->type_name(type);
}
int
afb_type_set_family(
 afb_type_t type,
 afb_type_t family
) {
 return afbBindingV4r1_itfptr->type_set_family(type, family);
}
int
afb_type_add_convert_to(
 afb_type_t type,
 afb_type_t to_type,
 afb_type_converter_t converter,
 void *closure
) {
 return afbBindingV4r1_itfptr->type_add_converter(type, to_type, converter, closure);
}
int
afb_type_add_convert_from(
 afb_type_t type,
 afb_type_t from_type,
 afb_type_converter_t converter,
 void *closure
) {
 return afbBindingV4r1_itfptr->type_add_converter(from_type, type, converter, closure);
}
int
afb_type_add_update_to(
 afb_type_t type,
 afb_type_t to_type,
 afb_type_updater_t updater,
 void *closure
) {
 return afbBindingV4r1_itfptr->type_add_updater(type, to_type, updater, closure);
}
int
afb_type_add_update_from(
 afb_type_t type,
 afb_type_t from_type,
 afb_type_updater_t updater,
 void *closure
) {
 return afbBindingV4r1_itfptr->type_add_updater(from_type, type, updater, closure);
}
int
afb_api_logmask(
 afb_api_t api
) {
 return afbBindingV4r1_itfptr->api_logmask(api);
}
const char *
afb_api_name(
 afb_api_t api
) {
 return afbBindingV4r1_itfptr->api_name(api);
}
void *
afb_api_get_userdata(
 afb_api_t api
) {
 return afbBindingV4r1_itfptr->api_get_userdata(api);
}
void *
afb_api_set_userdata(
 afb_api_t api,
 void *value
) {
 return afbBindingV4r1_itfptr->api_set_userdata(api, value);
}
int
afb_api_wants_log_level(
 afb_api_t api,
 int level
) {
 return ((afb_api_logmask(api)) & (1 << (level)));
}
void
afb_api_vverbose(
 afb_api_t api,
 int level,
 const char *file,
 int line,
 const char *func,
 const char *fmt,
 va_list args
) {
 afbBindingV4r1_itfptr->api_vverbose(api, level, file, line, func, fmt, args);
}
__attribute__((format(printf, 6, 7)))
void
afb_api_verbose(
 afb_api_t api,
 int level,
 const char *file,
 int line,
 const char *func,
 const char *fmt,
 ...
) {
 va_list args;
 __builtin_va_start(args,fmt);
 afbBindingV4r1_itfptr->api_vverbose(api, level, file, line, func, fmt, args);
 __builtin_va_end(args);
}
int
afb_api_broadcast_event(
 afb_api_t api,
 const char *name,
 unsigned nparams,
 afb_data_t const params[]
) {
 return afbBindingV4r1_itfptr->api_event_broadcast(api, name, nparams, params);
}
int
afb_api_require_api(
 afb_api_t api,
 const char *name,
 int initialized
) {
 return afbBindingV4r1_itfptr->api_require_api(api, name, initialized);
}
int
afb_api_new_event(
 afb_api_t api,
 const char *name,
 afb_event_t *event
) {
 return afbBindingV4r1_itfptr->api_new_event(api, name, event);
}
int
afb_api_add_verb(
 afb_api_t api,
 const char *verb,
 const char *info,
 afb_req_callback_t callback,
 void *vcbdata,
 const struct afb_auth *auth,
 uint32_t session,
 int glob
) {
 return afbBindingV4r1_itfptr->api_add_verb(api, verb, info, callback, vcbdata, auth, session, glob);
}
int
afb_api_del_verb(
 afb_api_t api,
 const char *verb,
 void **vcbdata
) {
 return afbBindingV4r1_itfptr->api_del_verb(api, verb, vcbdata);
}
void
afb_api_seal(
 afb_api_t api
) {
 afbBindingV4r1_itfptr->api_seal(api);
}
int
afb_api_set_verbs(
 afb_api_t api,
 const struct afb_verb_v4 *verbs
) {
 return afbBindingV4r1_itfptr->api_set_verbs(api, verbs);
}
int
afb_api_event_handler_add(
 afb_api_t api,
 const char *pattern,
 afb_event_handler_t callback,
 void *closure
) {
 return afbBindingV4r1_itfptr->api_event_handler_add(api, pattern, callback, closure);
}
int
afb_api_event_handler_del(
 afb_api_t api,
 const char *pattern,
 void **closure
) {
 return afbBindingV4r1_itfptr->api_event_handler_del(api, pattern, closure);
}
void
afb_api_call(
 afb_api_t api,
 const char *apiname,
 const char *verbname,
 unsigned nparams,
 afb_data_t const params[],
 afb_call_callback_t callback,
 void *closure
) {
 afbBindingV4r1_itfptr->api_call(api, apiname, verbname, nparams, params, callback, closure);
}
int
afb_api_call_sync(
 afb_api_t api,
 const char *apiname,
 const char *verbname,
 unsigned nparams,
 afb_data_t const params[],
 int *status,
 unsigned *nreplies,
 afb_data_t replies[]
) {
 return afbBindingV4r1_itfptr->api_call_sync(api,
   apiname, verbname, nparams, params,
   status, nreplies, replies);
}
int
afb_api_provide_class(
 afb_api_t api,
 const char *name
) {
 return afbBindingV4r1_itfptr->api_class_provide(api, name);
}
int
afb_api_require_class(
 afb_api_t api,
 const char *name
) {
 return afbBindingV4r1_itfptr->api_class_require(api, name);
}
int
afb_api_delete(
 afb_api_t api
) {
 return afbBindingV4r1_itfptr->api_delete(api);
}
struct json_object *
afb_api_settings(
 afb_api_t api
) {
 return afbBindingV4r1_itfptr->api_settings(api);
}
int
afb_create_api(
 afb_api_t *newapi,
 const char *apiname,
 const char *info,
 int noconcurrency,
 afb_api_callback_t mainctl,
 void *userdata
) {
 return afbBindingV4r1_itfptr->create_api(afbBindingV4root, newapi, apiname, info, noconcurrency, mainctl, userdata);
}
int
afb_job_post(
 long delayms,
 int timeout,
 void (*callback)(int signum, void *arg),
 void *argument,
 void *group
) {
 return afbBindingV4r1_itfptr->job_post(afbBindingV4root, delayms, timeout, callback, argument, group);
}
int
afb_job_abort(
 int jobid
) {
 return afbBindingV4r1_itfptr->job_abort(afbBindingV4root, jobid);
}
int
afb_alias_api(
 const char *name,
 const char *as_name
) {
 return afbBindingV4r1_itfptr->alias_api(afbBindingV4root, name, as_name);
}
int
afb_setup_shared_object(
 afb_api_t api,
 void *handle
) {
 return afbBindingV4r1_itfptr->setup_shared_object(api, handle);
}
int
afb_evfd_create(
 afb_evfd_t *efd,
 int fd,
 uint32_t events,
 afb_evfd_handler_t handler,
 void *closure,
 int autounref,
 int autoclose
) {
 return afbBindingV4r1_itfptr->evfd_create(
   efd, fd, events, handler, closure, autounref, autoclose);
}
afb_evfd_t
afb_evfd_addref(
 afb_evfd_t efd
) {
 return afbBindingV4r1_itfptr->evfd_addref(efd);
}
void
afb_evfd_unref(
 afb_evfd_t efd
) {
 return afbBindingV4r1_itfptr->evfd_unref(efd);
}
int
afb_evfd_get_fd(
 afb_evfd_t efd
) {
 return afbBindingV4r1_itfptr->evfd_get_fd(efd);
}
uint32_t
afb_evfd_get_events(
 afb_evfd_t efd
) {
 return afbBindingV4r1_itfptr->evfd_get_events(efd);
}
void
afb_evfd_set_events(
 afb_evfd_t efd,
 uint32_t events
) {
 return afbBindingV4r1_itfptr->evfd_set_events(efd, events);
}
int
afb_timer_create(
 afb_timer_t *timer,
 int absolute,
 time_t start_sec,
 unsigned start_ms,
 unsigned count,
 unsigned period_ms,
 unsigned accuracy_ms,
 afb_timer_handler_t handler,
 void *closure,
 int autounref
) {
 return afbBindingV4r1_itfptr->timer_create(
  timer, absolute, start_sec, start_ms,
  count, period_ms, accuracy_ms,
  handler, closure, autounref);
}
afb_timer_t
afb_timer_addref(
 afb_timer_t timer
) {
 return afbBindingV4r1_itfptr->timer_addref(timer);
}
void
afb_timer_unref(
 afb_timer_t timer
) {
 return afbBindingV4r1_itfptr->timer_unref(timer);
}
struct afb_verb_v4
{
 const char *verb;
 afb_req_callback_x4_t callback;
 const struct afb_auth *auth;
 const char *info;
 void *vcbdata;
 uint16_t session;
 uint16_t glob: 1;
};
struct afb_binding_v4
{
 const char *api;
 const char *specification;
 const char *info;
 const struct afb_verb_v4 *verbs;
 afb_api_callback_x4_t mainctl;
 void *userdata;
 const char *provide_class;
 const char *require_class;
 const char *require_api;
 unsigned noconcurrency: 1;
};
typedef struct afb_verb_v4 afb_verb_t;
typedef struct afb_binding_v4 afb_binding_t;
extern int afbBindingV4entry(afb_api_x4_t rootapi, afb_ctlid_t ctlid, afb_ctlarg_t ctlarg, void *userdata);
extern const struct afb_binding_v4 afbBindingV4;
 int afb_get_logmask()
{
 return afb_api_logmask(afbBindingV4root);
}
void afb_verbose(
int level,
const char *file,
int line,
const char *func,
const char *fmt,
...
) {
va_list args;
__builtin_va_start(
args
,
fmt
)
                   ;
afb_api_vverbose(afbBindingV4root, level, file, line, func, fmt, args);
__builtin_va_end(
args
)
           ;
}
