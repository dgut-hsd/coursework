package org.example.saasplatform.common;

import com.alibaba.fastjson2.annotation.JSONField;
import com.fasterxml.jackson.annotation.JsonInclude;

/**
 * Unified API response wrapper.
 * <pre>{@code
 * { "code": 200, "message": "success", "data": {...}, "timestamp": 1720000000000 }
 * }</pre>
 */
@JsonInclude(JsonInclude.Include.NON_NULL)
public class Result<T> {

    private Integer code;
    private String message;
    private T data;
    private Long timestamp;

    private Result() {
        this.timestamp = System.currentTimeMillis();
    }

    // ── Success ─────────────────────────────────────────────

    public static <T> Result<T> success(T data) {
        Result<T> r = new Result<>();
        r.code = 200;
        r.message = "success";
        r.data = data;
        return r;
    }

    public static <T> Result<T> success() {
        return success(null);
    }

    // ── Error ───────────────────────────────────────────────

    public static <T> Result<T> error(Integer code, String message) {
        Result<T> r = new Result<>();
        r.code = code;
        r.message = message;
        return r;
    }

    public static <T> Result<T> error(String message) {
        return error(500, message);
    }

    // ── Getters / Setters ───────────────────────────────────

    public Integer getCode() { return code; }
    public void setCode(Integer code) { this.code = code; }

    public String getMessage() { return message; }
    public void setMessage(String message) { this.message = message; }

    public T getData() { return data; }
    public void setData(T data) { this.data = data; }

    public Long getTimestamp() { return timestamp; }
    public void setTimestamp(Long timestamp) { this.timestamp = timestamp; }
}
