#include <gtk/gtk.h>
#include <stdbool.h>

void pop_analytics_panel_init(void);

void pop_analytics_panel_attach(GtkContainer *container, GtkWindow *window);

void pop_analytics_panel_eula_attach(GtkContainer *container);

void pop_analytics_panel_initial_setup_attach(
    GtkContainer *container,
    void (*callback_fn)(void *, bool),
    void * data
);

void pop_analytics_panel_summary_attach(GtkContainer *container);

gboolean pop_analytics_panel_should_show(void);
