/* Optional native-smoke preload; uses the installed official RenderDoc header.
 * Normal editor launches never load this library or install its polling thread.
 */
#define _POSIX_C_SOURCE 200809L
#include <ctype.h>
#include <dlfcn.h>
#include <pthread.h>
#include <stdatomic.h>
#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>
#include <unistd.h>
#include "renderdoc_app.h"

static pthread_t worker;
static atomic_bool stopping;
static int started;
static RENDERDOC_API_1_6_0 *api;

static void *capture_requests(void *unused)
{
    (void)unused;
    const char *control = getenv("ZE_RENDERDOC_CONTROL");
    const char *root = getenv("ZE_RENDERDOC_CAPTURE_ROOT");
    char previous[128] = "";
    const struct timespec delay = { .tv_sec = 0, .tv_nsec = 100000000 };
    while (!atomic_load(&stopping)) {
        char label[128] = "";
        FILE *request = fopen(control, "r");
        if (request) {
            if (!fgets(label, sizeof(label), request)) label[0] = '\0';
            fclose(request);
        }
        label[strcspn(label, "\r\n")] = '\0';
        if (label[0] && strcmp(label, previous)) {
            strcpy(previous, label);
            for (char *p = label; *p; ++p)
                if (!isalnum((unsigned char)*p) && *p != '-' && *p != '_') *p = '_';
            char path[4096];
            int size = snprintf(path, sizeof(path), "%s/%s", root, label);
            if (size > 0 && (size_t)size < sizeof(path)) {
                api->SetCaptureFilePathTemplate(path);
                api->TriggerCapture();
                fprintf(stderr, "renderdoc_api pid=%ld trigger=%s\n", (long)getpid(), label);
            }
        }
        nanosleep(&delay, NULL);
    }
    return NULL;
}

__attribute__((constructor)) static void start_capture_helper(int argc, char **argv)
{
    for (int i = 1; i < argc; ++i)
        if (!strcmp(argv[i], "--zircon-startup-splash-child")) return;
    if (!getenv("ZE_RENDERDOC_CONTROL") || !getenv("ZE_RENDERDOC_CAPTURE_ROOT")) return;
    pRENDERDOC_GetAPI get_api = (pRENDERDOC_GetAPI)dlsym(RTLD_DEFAULT, "RENDERDOC_GetAPI");
    if (!get_api || !get_api(eRENDERDOC_API_Version_1_6_0, (void **)&api)) {
        fprintf(stderr, "renderdoc_api official API unavailable\n");
        return;
    }
    api->SetCaptureOptionU32(eRENDERDOC_Option_APIValidation, 0);
    api->SetCaptureOptionU32(eRENDERDOC_Option_CaptureCallstacks, 0);
    started = pthread_create(&worker, NULL, capture_requests, NULL) == 0;
    fprintf(stderr, "renderdoc_api pid=%ld helper_started=%d\n", (long)getpid(), started);
}

__attribute__((destructor)) static void stop_capture_helper(void)
{
    if (started) {
        atomic_store(&stopping, 1);
        pthread_join(worker, NULL);
    }
}
