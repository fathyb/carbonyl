#ifndef CARBONYL_SRC_BROWSER_RENDERER_H_
#define CARBONYL_SRC_BROWSER_RENDERER_H_

#include <cstdint>
#include <functional>

#include "base/functional/callback.h"
#include "carbonyl/src/browser/export.h"
#include "ui/gfx/geometry/rect_f.h"

extern "C" {

struct carbonyl_renderer;
struct carbonyl_renderer_browser_delegate {
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

struct CARBONYL_RENDERER_EXPORT Text {
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

class CARBONYL_RENDERER_EXPORT Renderer {
public:
    static void Main();
    static Renderer* GetCurrent();

    gfx::Size GetSize();

    gfx::Size Resize();
    void StartRenderer();
    void Listen(const struct carbonyl_renderer_browser_delegate* delegate);
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
    Renderer(struct carbonyl_renderer* ptr);

    struct carbonyl_renderer* ptr_;
};

}

#endif  // CARBONYL_SRC_BROWSER_RENDERER_H_
