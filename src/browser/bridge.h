#ifndef CARBONYL_SRC_BROWSER_BRIDGE_H_
#define CARBONYL_SRC_BROWSER_BRIDGE_H_

#include <cstdint>
#include <functional>

#include "base/functional/callback.h"
#include "ui/gfx/geometry/rect_f.h"

extern "C" {

struct carbonyl_bridge;
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

} /* end extern "C" */

namespace carbonyl {

struct Text {
    Text(
        std::string text,
        gfx::RectF rect,
        uint32_t color
    ):
        text(text),
        rect(rect),
        color(color)
    {}

    std::string text;
    gfx::RectF rect;
    uint32_t color;
};

class Bridge {
public:
    static void Main();
    static Bridge* GetCurrent();
    static bool BitmapMode();

    gfx::Size GetSize();
    float GetDPI();

    gfx::Size Resize();
    void StartRenderer();
    void Listen(const struct carbonyl_bridge_browser_delegate* delegate);
    void PushNav(const std::string& url, bool can_go_back, bool can_go_forward);
    void SetTitle(const std::string& title);
    void DrawText(const std::vector<Text>& text);
    void DrawBitmap(
        const unsigned char* pixels,
        const gfx::Size& size,
        const gfx::Rect& damage,
        base::OnceCallback<void()> callback
    );

private:
    Bridge(struct carbonyl_bridge* ptr);

    struct carbonyl_bridge* ptr_;
};

}

#endif  // CARBONYL_SRC_BROWSER_BRIDGE_H_
