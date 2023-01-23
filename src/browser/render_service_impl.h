#ifndef CARBONYL_SRC_BROWSER_RENDER_SERVICE_IMPL_H_
#define CARBONYL_SRC_BROWSER_RENDER_SERVICE_IMPL_H_

#include "carbonyl/src/browser/carbonyl.mojom.h"
#include "mojo/public/cpp/bindings/pending_receiver.h"
#include "mojo/public/cpp/bindings/receiver.h"

namespace carbonyl {

class CarbonylRenderServiceImpl: public mojom::CarbonylRenderService {
 public:
  explicit CarbonylRenderServiceImpl(mojo::PendingReceiver<mojom::CarbonylRenderService> receiver);
  CarbonylRenderServiceImpl(const CarbonylRenderServiceImpl&) = delete;
  CarbonylRenderServiceImpl& operator=(const CarbonylRenderServiceImpl&) = delete;

  ~CarbonylRenderServiceImpl() override;

  // carbonyl::mojom::CarbonylRenderService:
  void DrawText(std::vector<mojom::TextDataPtr> data) override;

 private:
  mojo::Receiver<mojom::CarbonylRenderService> receiver_;
};

}  // namespace carbonyl

#endif  // CARBONYL_SRC_BROWSER_RENDER_SERVICE_IMPL_H_