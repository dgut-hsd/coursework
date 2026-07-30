package org.example.saasplatform.client;

import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.springframework.stereotype.Component;

import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;

/**
 * Mock Rust AI engine client.
 * <p>
 * In production: uses {@code java.net.http.HttpClient} or {@code RestClient}
 * to call Rust engine REST API on port 3001.
 * <p>
 * In demo: simulates audit processing with Thread.sleep + mock results.
 */
@Component
public class RustApiClient {

    private static final Logger log = LoggerFactory.getLogger(RustApiClient.class);

    /**
     * Simulate calling the Rust AI engine to audit a document.
     * Returns mock findings after a simulated processing delay.
     */
    public Map<String, Object> simulateAudit(Long taskId, List<String> checks) {
        log.info("RustApiClient: starting simulated audit for taskId={}, checks={}", taskId, checks);

        // Simulate Rust engine processing time
        try {
            Thread.sleep(3000);
        } catch (InterruptedException e) {
            Thread.currentThread().interrupt();
            log.warn("Audit simulation interrupted for taskId={}, returning partial result", taskId);
        }

        // Build mock result
        Map<String, Object> result = new LinkedHashMap<>();
        result.put("taskId", taskId);

        // Random score 75-99
        int score = 75 + (int) (Math.random() * 25);
        result.put("score", score);

        // Mock findings
        List<Map<String, Object>> findings = List.of(
                Map.of("severity", "HIGH",
                        "check", "qualification",
                        "message", "资质证书即将过期，剩余有效期不足30天"),
                Map.of("severity", "MEDIUM",
                        "check", "pricing",
                        "message", "报价明细表存在明显计算错误：第3行单价与数量乘积不匹配"),
                Map.of("severity", "LOW",
                        "check", "safety",
                        "message", "安全生产许可证信息已更新，建议核实最新版本")
        );
        result.put("findings", findings);

        log.info("RustApiClient: audit completed for taskId={}, score={}", taskId, score);
        return result;
    }
}
