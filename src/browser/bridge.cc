#include "carbonyl/src/browser/bridge.h"

#include <memory>
#include <iostream>
#include <stdio.h>

#include "third_party/skia/include/core/SkColor.h"

extern "C" {

void* carbonyl_renderer_create();
void carbonyl_renderer_clear_text(void* renderer);
void carbonyl_input_listen(void* renderer, void* delegate);
void carbonyl_renderer_draw_text(
    void* renderer,
    const char* utf8,
    const struct carbonyl_bridge_rect* rect,
    const struct carbonyl_bridge_color* color
);
void carbonyl_renderer_draw_background(
    void* renderer,
    const unsigned char* pixels,
    size_t pixels_size,
    const struct carbonyl_bridge_rect* rect
);

}

namespace carbonyl {

namespace {
    static std::unique_ptr<Renderer> globalInstance;
}

Renderer::Renderer(void* ptr): ptr_(ptr) {}

Renderer* Renderer::Main() {
    if (!globalInstance) {
        globalInstance = std::make_unique<Renderer>(
            carbonyl_renderer_create()
        );
    }

    return globalInstance.get();
}
void Renderer::Listen(void* delegate) {
    carbonyl_input_listen(ptr_, delegate);
}

void Renderer::ClearText() {
    carbonyl_renderer_clear_text(ptr_);
}

void Renderer::DrawText(const std::string& text, const gfx::RectF& bounds, uint32_t sk_color) {
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
