// Copyright 2025 Mongoose Browser Project
// Use of this source code is governed by an MIT-style license.

#include "content/browser/mongoose_url_loader_interceptor.h"

#include "base/bind.h"
#include "base/logging.h"
#include "base/memory/ref_counted.h"
#include "content/public/browser/browser_thread.h"
#include "net/base/net_errors.h"
#include "services/network/public/cpp/resource_request.h"
#include "services/network/public/cpp/single_request_url_loader_factory.h"
#include "services/network/public/mojom/url_loader.mojom.h"
#include "services/network/public/mojom/url_loader_completion_status.mojom.h"

namespace content {

MongooseURLLoaderInterceptor::MongooseURLLoaderInterceptor() {
  // TODO: Initialize Mojo connection to Rust MongooseOrchestrator
  // This should be done during browser process startup
  //
  // Example:
  // mojo::PendingRemote<mongoose::mojom::MongooseRenderService> pending_remote;
  // // Bind pending_remote to Rust side
  // mongoose_service_.Bind(std::move(pending_remote));

  LOG(INFO) << "Mongoose URL Loader Interceptor initialized";
}

MongooseURLLoaderInterceptor::~MongooseURLLoaderInterceptor() {
  LOG(INFO) << "Mongoose URL Loader Interceptor shutdown"
            << " (blocked " << blocked_requests_
            << " / " << total_requests_ << " requests)";
}

void MongooseURLLoaderInterceptor::SetMongooseServiceForTesting(
    mojo::Remote<mongoose::mojom::MongooseRenderService> service) {
  mongoose_service_ = std::move(service);
}

bool MongooseURLLoaderInterceptor::ShouldBlockRequest(
    const network::ResourceRequest& request,
    std::string* matched_filter) {

  if (!mongoose_service_.is_bound()) {
    // Mojo connection not established - allow all requests
    VLOG(1) << "Mongoose service not bound, allowing request";
    return false;
  }

  // Create NetworkRequestData for Mojo call
  auto request_data = mongoose::mojom::NetworkRequestData::New();
  request_data->url = request.url;

  // Extract initiator origin (source URL)
  if (request.request_initiator.has_value()) {
    request_data->initiator = request.request_initiator.value();
  } else {
    // Use empty origin as fallback
    request_data->initiator = url::Origin();
  }

  // Convert Chromium resource type to adblock type
  // See: third_party/blink/public/mojom/loader/resource_type.mojom
  request_data->resource_type = static_cast<int32_t>(request.resource_type);

  // Make SYNCHRONOUS Mojo call to Rust privacy engine
  //
  // CRITICAL: This call MUST complete in microseconds to avoid blocking
  // the browser's network stack. The adblock-rust engine uses in-memory
  // tries for O(log n) lookups, so this should be fast.
  mongoose::mojom::BlockResultPtr result;
  bool success = mongoose_service_->CheckNetworkRequest(
      std::move(request_data), &result);

  if (!success) {
    LOG(ERROR) << "Mojo call to CheckNetworkRequest failed";
    return false;  // Allow on error (fail open)
  }

  if (result->should_block) {
    *matched_filter = result->matched_filter.value_or("unknown");
    return true;
  }

  return false;
}

void MongooseURLLoaderInterceptor::MaybeCreateLoader(
    const network::ResourceRequest& tentative_resource_request,
    BrowserContext* browser_context,
    LoaderCallback callback,
    FallbackCallback fallback_callback) {

  DCHECK_CURRENTLY_ON(BrowserThread::UI);

  total_requests_++;

  // Check if the request should be blocked
  std::string matched_filter;
  if (ShouldBlockRequest(tentative_resource_request, &matched_filter)) {
    blocked_requests_++;

    LOG(INFO) << "Mongoose: Blocking request to "
              << tentative_resource_request.url.spec()
              << " (filter: " << matched_filter << ")";

    // Create a URLLoader that immediately fails the request
    //
    // This uses SingleRequestURLLoaderFactory to create a one-shot loader
    // that calls the client's OnComplete callback with ERR_BLOCKED_BY_CLIENT.
    auto loader_factory = base::MakeRefCounted<
        network::SingleRequestURLLoaderFactory>(base::BindOnce(
        [](const network::ResourceRequest& request,
           mojo::PendingReceiver<network::mojom::URLLoader> receiver,
           mojo::PendingRemote<network::mojom::URLLoaderClient> client) {
          // Bind the client remote
          mojo::Remote<network::mojom::URLLoaderClient> client_remote(
              std::move(client));

          // Send ERR_BLOCKED_BY_CLIENT status
          network::URLLoaderCompletionStatus status;
          status.error_code = net::ERR_BLOCKED_BY_CLIENT;
          status.exists_in_cache = false;
          status.completion_time = base::TimeTicks::Now();

          // Notify the client that the request is blocked
          client_remote->OnComplete(status);
        }));

    // Return the blocking loader via callback
    std::move(callback).Run(std::move(loader_factory));
    return;
  }

  // Request is allowed - use default loader
  VLOG(2) << "Mongoose: Allowing request to "
          << tentative_resource_request.url.spec();

  std::move(fallback_callback).Run();
}

}  // namespace content
