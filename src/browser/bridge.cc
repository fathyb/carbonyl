#include "carbonyl/src/browser/bridge.h"

#include <memory>
#include <iostream>
#include <stdio.h>

#include "third_party/skia/include/core/SkColor.h"

extern "C" {

struct carbonyl_renderer* carbonyl_renderer_create();
void carbonyl_renderer_set_url(struct carbonyl_renderer* renderer, const char* url);
void carbonyl_renderer_set_title(struct carbonyl_renderer* renderer, const char* title);
void carbonyl_renderer_clear_text(struct carbonyl_renderer* renderer);
void carbonyl_input_listen(struct carbonyl_renderer* renderer, const struct carbonyl_bridge_browser_delegate* delegate);
void carbonyl_renderer_draw_text(
    struct carbonyl_renderer* renderer,
    const char* title,
    const struct carbonyl_bridge_rect* rect,
    const struct carbonyl_bridge_color* color
);
void carbonyl_renderer_draw_background(
    struct carbonyl_renderer* renderer,
    const unsigned char* pixels,
    size_t pixels_size,
    const struct carbonyl_bridge_rect* rect
);

}

namespace carbonyl {

namespace {
    static std::unique_ptr<Renderer> globalInstance;
}

Renderer::Renderer(struct carbonyl_renderer* ptr): ptr_(ptr) {}

Renderer* Renderer::Main() {
    if (!globalInstance) {
        globalInstance = std::unique_ptr<Renderer>(
            new Renderer(carbonyl_renderer_create())
        );
    }

    return globalInstance.get();
}
void Renderer::Listen(const struct carbonyl_bridge_browser_delegate* delegate) {
    carbonyl_input_listen(ptr_, delegate);
}

void Renderer::SetURL(const std::string& url) {
    if (!url.size()) {
        return;
    }

    carbonyl_renderer_set_url(ptr_, url.c_str());
}

void Renderer::SetTitle(const std::string& title) {
    if (!title.size()) {
        return;
    }

    carbonyl_renderer_set_title(ptr_, title.c_str());
}

void Renderer::ClearText() {
    carbonyl_renderer_clear_text(ptr_);
}

void Renderer::DrawText(const std::string& text, const gfx::RectF& bounds, uint32_t sk_color) {
    if (!text.size()) {
        return;
    }

    struct carbonyl_bridge_rect rect;
    struct carbonyl_bridge_color color;

    rect.origin.x = bounds.x();
    rect.origin.y = bounds.y();
    rect.size.width = bounds.width();
    rect.size.height = bounds.height();

    color.r = SkColorGetR(sk_color);
    color.g = SkColorGetG(sk_color);
    color.b = SkColorGetB(sk_color);

    carbonyl_renderer_draw_text(ptr_, text.c_str(), &rect, &color);
}

void Renderer::DrawBackgrond(const unsigned char* pixels, size_t pixels_size, const gfx::Rect& bounds) {
    struct carbonyl_bridge_rect rect;

    rect.origin.x = bounds.x();
    rect.origin.y = bounds.y();
    rect.size.width = bounds.width();
    rect.size.height = bounds.height();

    carbonyl_renderer_draw_background(ptr_, pixels, pixels_size, &rect);
}

}
