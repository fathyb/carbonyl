#ifndef CARBONYL_SRC_BROWSER_BRIDGE_H_
#define CARBONYL_SRC_BROWSER_BRIDGE_H_

#include <cstdint>

#include "ui/gfx/geometry/rect_f.h"

extern "C" {

struct carbonyl_renderer;

struct carbonyl_bridge_size {
    unsigned int width;
    unsigned int height;
};
struct carbonyl_bridge_point {
    unsigned int x;
    unsigned int y;
};
struct carbonyl_bridge_rect {
    struct carbonyl_bridge_point origin;
    struct carbonyl_bridge_size size;
};
struct carbonyl_bridge_color {
    uint8_t r;
    uint8_t g;
    uint8_t b;
};
struct carbonyl_bridge_browser_delegate {
    void (*shutdown) ();
    void (*refresh) ();
    void (*go_to) (const char* url);
    void (*go_back) ();
    void (*go_forward) ();
    void (*scroll) (int);
    void (*key_press) (char);
    void (*mouse_up) (unsigned int, unsigned int);
    void (*mouse_down) (unsigned int, unsigned int);
    void (*mouse_move) (unsigned int, unsigned int);
    void (*post_task) (void (*)(void*), void*);
};

void carbonyl_shell_main();

} /* end extern "C" */

namespace carbonyl {

class Renderer {
public:
    static Renderer* Main();
    static gfx::Size GetSize();

    void Resize();
    void Listen(const struct carbonyl_bridge_browser_delegate* delegate);
    void PushNav(const std::string& url, bool can_go_back, bool can_go_forward);
    void SetTitle(const std::string& title);
    void ClearText();
    void DrawText(const std::string& text, const gfx::RectF& bounds, uint32_t color);
    void DrawBackgrond(const unsigned char* pixels, size_t pixels_size, const gfx::Rect& bounds);

private:
    Renderer(struct carbonyl_renderer* ptr);

    struct carbonyl_renderer* ptr_;
};

}

#endif  // CARBONYL_SRC_BROWSER_BRIDGE_H_
