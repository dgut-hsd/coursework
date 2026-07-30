package org.example.saasplatform.client;

import org.example.saasplatform.sse.SseHub;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Component;

/**
 * SSE relay from Rust engine to Java SseHub.
 * <p>
 * In production: connects to Rust engine's SSE endpoint ({@code /api/review/{id}/stream}),
 * parses SSE events, and relays them to {@link SseHub} for delivery to frontend clients.
 * <p>
 * In demo: not directly used — AuditEngineServiceImpl calls SseHub directly.
 */
@Component
public class RustSseClient {

    private static final Logger log = LoggerFactory.getLogger(RustSseClient.class);

    @Autowired
    private SseHub sseHub;

    /**
     * In production: connect to Rust SSE stream and relay events.
     * <pre>{@code
     * HttpClient client = HttpClient.newHttpClient();
     * HttpRequest request = HttpRequest.newBuilder()
     *     .uri(URI.create(rustBaseUrl + "/api/review/" + rustTaskId + "/stream"))
     *     .GET().build();
     * client.sendAsync(request, BodyHandlers.ofLines())
     *     .thenAccept(response -> {
     *         response.body().forEach(line -> {
     *             if (line.startsWith("data:")) {
     *                 AuditTaskEvent event = parseRustEvent(line);
     *                 sseHub.sendEvent(taskId, event.getType(), event.getData());
     *             }
     *         });
     *     });
     * }</pre>
     */
    public void relayFromRust(Long taskId, String rustSseUrl) {
        log.info("RustSseClient: relay placeholder — would connect to: {}", rustSseUrl);
        // Production implementation: see javadoc above
    }
}
