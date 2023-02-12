#include "carbonyl/src/browser/bridge.h"

#include <memory>
#include <iostream>
#include <stdio.h>

#include "base/functional/callback.h"
#include "ui/gfx/geometry/rect_f.h"
#include "third_party/skia/include/core/SkColor.h"
#include "third_party/skia/src/core/SkBitmapDevice.h"

extern "C" {

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
struct carbonyl_bridge_text {
    const char* text;
    carbonyl_bridge_rect rect;
    carbonyl_bridge_color color;
};

void carbonyl_shell_main();
bool carbonyl_shell_bitmap_mode();

struct carbonyl_bridge* carbonyl_bridge_create();
void carbonyl_bridge_start_renderer(struct carbonyl_bridge* bridge);
void carbonyl_bridge_resize(struct carbonyl_bridge* bridge);
float carbonyl_bridge_get_dpi(struct carbonyl_bridge* bridge);
struct carbonyl_bridge_size carbonyl_bridge_get_size(struct carbonyl_bridge* bridge);
void carbonyl_bridge_push_nav(struct carbonyl_bridge* bridge, const char* url, bool can_go_back, bool can_go_forward);
void carbonyl_bridge_set_title(struct carbonyl_bridge* bridge, const char* title);
void carbonyl_bridge_clear_text(struct carbonyl_bridge* bridge);
void carbonyl_bridge_listen(struct carbonyl_bridge* bridge, const struct carbonyl_bridge_browser_delegate* delegate);
void carbonyl_bridge_draw_text(
    struct carbonyl_bridge* bridge,
    const struct carbonyl_bridge_text* text,
    size_t text_size
);
void carbonyl_bridge_draw_bitmap(
    struct carbonyl_bridge* bridge,
    const unsigned char* pixels,
    const struct carbonyl_bridge_size size,
    const struct carbonyl_bridge_rect rect,
    void (*callback) (void*),
    void* callback_data
);

}

namespace carbonyl {

namespace {
    static std::unique_ptr<Bridge> globalInstance;
}

Bridge::Bridge(struct carbonyl_bridge* ptr): ptr_(ptr) {}

void Bridge::Main() {
    carbonyl_shell_main();
}

Bridge* Bridge::GetCurrent() {
    if (!globalInstance) {
        globalInstance = std::unique_ptr<Bridge>(
            new Bridge(carbonyl_bridge_create())
        );
    }

    return globalInstance.get();
}

namespace {
    thread_local int bitmap_mode = -1;
}

bool Bridge::BitmapMode() {
    if (bitmap_mode == -1) {
        bitmap_mode = carbonyl_shell_bitmap_mode();

        if (!bitmap_mode) {
            SkBitmapDevice::DisableTextRendering();
        }
    }

    return bitmap_mode;
}

float Bridge::GetDPI() {
    return carbonyl_bridge_get_dpi(ptr_);
}

void Bridge::StartRenderer() {
    carbonyl_bridge_start_renderer(ptr_);
}

gfx::Size Bridge::GetSize() {
    auto size = carbonyl_bridge_get_size(ptr_);

    return gfx::Size(size.width, size.height);
}

gfx::Size Bridge::Resize() {
    carbonyl_bridge_resize(ptr_);

    return GetSize();
}

void Bridge::Listen(const struct carbonyl_bridge_browser_delegate* delegate) {
    carbonyl_bridge_listen(ptr_, delegate);
}

void Bridge::PushNav(const std::string& url, bool can_go_back, bool can_go_forward) {
    if (!url.size()) {
        return;
    }

    carbonyl_bridge_push_nav(ptr_, url.c_str(), can_go_back, can_go_forward);
}

void Bridge::SetTitle(const std::string& title) {
    if (!title.size()) {
        return;
    }

    carbonyl_bridge_set_title(ptr_, title.c_str());
}

void Bridge::DrawText(const std::vector<Text>& text) {
    struct carbonyl_bridge_text data[text.size()];

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

    carbonyl_bridge_draw_text(ptr_, data, text.size());
}

void Bridge::DrawBitmap(
    const unsigned char* pixels,
    const gfx::Size& pixels_size,
    const gfx::Rect& damage,
    base::OnceCallback<void()> callback
) {
    auto* box = new base::OnceCallback<void()>(std::move(callback));

    carbonyl_bridge_draw_bitmap(
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
