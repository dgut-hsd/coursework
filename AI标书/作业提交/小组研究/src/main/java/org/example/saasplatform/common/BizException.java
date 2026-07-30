package org.example.saasplatform.common;

/**
 * Business exception with error code.
 * Caught by {@link GlobalExceptionHandler} and serialized to {@link Result}.
 */
public class BizException extends RuntimeException {

    private final int code;

    public BizException(int code, String message) {
        super(message);
        this.code = code;
    }

    public BizException(String message) {
        this(400, message);
    }

    public int getCode() {
        return code;
    }
}
