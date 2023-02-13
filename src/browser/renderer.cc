#include "carbonyl/src/browser/renderer.h"

#include <memory>
#include <iostream>
#include <stdio.h>

#include "base/functional/callback.h"
#include "carbonyl/src/browser/bridge.h"
#include "ui/gfx/geometry/rect_f.h"
#include "third_party/skia/include/core/SkColor.h"

extern "C" {

struct carbonyl_renderer_size {
    unsigned int width;
    unsigned int height;
};
struct carbonyl_renderer_point {
    unsigned int x;
    unsigned int y;
};
struct carbonyl_renderer_rect {
    struct carbonyl_renderer_point origin;
    struct carbonyl_renderer_size size;
};
struct carbonyl_renderer_color {
    uint8_t r;
    uint8_t g;
    uint8_t b;
};
struct carbonyl_renderer_text {
    const char* text;
    carbonyl_renderer_rect rect;
    carbonyl_renderer_color color;
};

void carbonyl_bridge_main();

struct carbonyl_renderer* carbonyl_renderer_create();
void carbonyl_renderer_start(struct carbonyl_renderer* renderer);
void carbonyl_renderer_resize(struct carbonyl_renderer* renderer);
struct carbonyl_renderer_size carbonyl_renderer_get_size(struct carbonyl_renderer* renderer);
void carbonyl_renderer_push_nav(struct carbonyl_renderer* renderer, const char* url, bool can_go_back, bool can_go_forward);
void carbonyl_renderer_set_title(struct carbonyl_renderer* renderer, const char* title);
void carbonyl_renderer_clear_text(struct carbonyl_renderer* renderer);
void carbonyl_renderer_listen(struct carbonyl_renderer* renderer, const struct carbonyl_renderer_browser_delegate* delegate);
void carbonyl_renderer_draw_text(
    struct carbonyl_renderer* renderer,
    const struct carbonyl_renderer_text* text,
    size_t text_size
);
void carbonyl_renderer_draw_bitmap(
    struct carbonyl_renderer* renderer,
    const unsigned char* pixels,
    const struct carbonyl_renderer_size size,
    const struct carbonyl_renderer_rect rect,
    void (*callback) (void*),
    void* callback_data
);

}

namespace carbonyl {

namespace {
    static std::unique_ptr<Renderer> globalInstance;
}

Renderer::Renderer(struct carbonyl_renderer* ptr): ptr_(ptr) {}

void Renderer::Main() {
    carbonyl_bridge_main();
}

Renderer* Renderer::GetCurrent() {
    if (!globalInstance) {
        globalInstance = std::unique_ptr<Renderer>(
            new Renderer(carbonyl_renderer_create())
        );
    }

    return globalInstance.get();
}

void Renderer::StartRenderer() {
    carbonyl_renderer_start(ptr_);
}

gfx::Size Renderer::GetSize() {
    auto size = carbonyl_renderer_get_size(ptr_);

    return gfx::Size(size.width, size.height);
}

gfx::Size Renderer::Resize() {
    carbonyl_renderer_resize(ptr_);
    Bridge::Resize();

    return GetSize();
}

void Renderer::Listen(const struct carbonyl_renderer_browser_delegate* delegate) {
    carbonyl_renderer_listen(ptr_, delegate);
}

void Renderer::PushNav(const std::string& url, bool can_go_back, bool can_go_forward) {
    if (!url.size()) {
        return;
    }

    carbonyl_renderer_push_nav(ptr_, url.c_str(), can_go_back, can_go_forward);
}

void Renderer::SetTitle(const std::string& title) {
    if (!title.size()) {
        return;
    }

    carbonyl_renderer_set_title(ptr_, title.c_str());
}

void Renderer::DrawText(const std::vector<Text>& text) {
    struct carbonyl_renderer_text data[text.size()];

    for (size_t i = 0; i < text.size(); i++) {
        data[i].text = text[i].text.c_str();
        data[i].color.r = SkColorGetR(text[i].color);
        data[i].color.g = SkColorGetG(text[i].color);
        data[i].color.b = SkColorGetB(text[i].color);
        data[i].rect.origin.x = text[i].rect.x();
        data[i].rect.origin.y = text[i].rect.y();
        data[i].rect.size.width = std::ceil(text[i].rect.width());
        data[i].rect.size.height = std::ceil(text[i].rect.height());
    }

    carbonyl_renderer_draw_text(ptr_, data, text.size());
}

void Renderer::DrawBitmap(
    const unsigned char* pixels,
    const gfx::Size& pixels_size,
    const gfx::Rect& damage,
    base::OnceCallback<void()> callback
) {
    auto* box = new base::OnceCallback<void()>(std::move(callback));

    carbonyl_renderer_draw_bitmap(
        ptr_,
        pixels,
        {
            .width = (unsigned int)pixels_size.width(),
            .height = (unsigned int)pixels_size.height(),
        },
        {
            .origin = {
                .x = (unsigned int)damage.x(),
                .y = (unsigned int)damage.y(),
            },
            .size = {
                .width = (unsigned int)damage.width(),
                .height = (unsigned int)damage.height(),
            },
        },
        [](void* box) {
            auto* ptr = static_cast<base::OnceCallback<void()>*>(box);

            std::move(*ptr).Run();
            delete ptr;
        },
        box
    );
}

}
