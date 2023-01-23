#include "carbonyl/src/browser/render_service_impl.h"

#include <iostream>

#include "carbonyl/src/browser/bridge.h"

namespace carbonyl {

CarbonylRenderServiceImpl::CarbonylRenderServiceImpl(
    mojo::PendingReceiver<mojom::CarbonylRenderService> receiver):
    receiver_(this, std::move(receiver))
{}

CarbonylRenderServiceImpl::~CarbonylRenderServiceImpl() = default;

void CarbonylRenderServiceImpl::DrawText(std::vector<mojom::TextDataPtr> data) {
    auto* renderer = Renderer::Main();

    renderer->ClearText();

    for (auto& text: data) {
        renderer->DrawText(text->contents, text->bounds, text->color);
    }
}

}
