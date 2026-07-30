package org.example.saasplatform.config;

import com.baomidou.mybatisplus.annotation.DbType;
import com.baomidou.mybatisplus.extension.plugins.MybatisPlusInterceptor;
import com.baomidou.mybatisplus.extension.plugins.inner.OptimisticLockerInnerInterceptor;
import com.baomidou.mybatisplus.extension.plugins.inner.PaginationInnerInterceptor;
import com.baomidou.mybatisplus.extension.plugins.inner.TenantLineInnerInterceptor;
import net.sf.jsqlparser.expression.Expression;
import net.sf.jsqlparser.expression.LongValue;
import org.example.saasplatform.common.BaseContext;
import org.example.saasplatform.common.BizException;
import org.mybatis.spring.annotation.MapperScan;
import org.springframework.context.annotation.Bean;
import org.springframework.context.annotation.Configuration;

import java.util.List;

/**
 * MyBatis-Plus configuration: multi-tenancy + pagination + optimistic locking.
 * <p>
 * - TenantLineInnerInterceptor: auto-injects {@code WHERE tenant_id = ?} into every query
 * - PaginationInnerInterceptor: physical pagination (MySQL LIMIT)
 * - OptimisticLockerInnerInterceptor: {@code @Version} field support
 */
@Configuration
@MapperScan("org.example.saasplatform.mapper")
public class MybatisPlusConfig {

    @Bean
    public MybatisPlusInterceptor mybatisPlusInterceptor() {
        MybatisPlusInterceptor interceptor = new MybatisPlusInterceptor();

        // 1. Multi-tenancy interceptor — automatically appends WHERE tenant_id = ?
        interceptor.addInnerInterceptor(new TenantLineInnerInterceptor(new CustomTenantLineHandler()));

        // 2. Pagination interceptor
        interceptor.addInnerInterceptor(new PaginationInnerInterceptor(DbType.MYSQL));

        // 3. Optimistic locking interceptor (@Version)
        interceptor.addInnerInterceptor(new OptimisticLockerInnerInterceptor());

        return interceptor;
    }

    /**
     * Custom tenant handler: reads tenantId from BaseContext ThreadLocal.
     */
    static class CustomTenantLineHandler implements com.baomidou.mybatisplus.extension.plugins.handler.TenantLineHandler {

        /**
         * Tables that should NOT be tenant-filtered.
         * - sys_config: global configuration
         * - sys_user: login happens before tenant context exists
         * - audit_task_event: no tenant_id column (scoped by task_id)
         * - flyway_schema_history: Flyway internal table
         */
        private static final List<String> IGNORE_TABLES = List.of(
                "sys_config", "sys_user", "audit_task_event", "flyway_schema_history"
        );

        @Override
        public Expression getTenantId() {
            Long tenantId = BaseContext.getCurrentTenantId();
            if (tenantId == null) {
                throw new BizException(401, "未登录或租户信息缺失");
            }
            return new LongValue(tenantId);
        }

        @Override
        public String getTenantIdColumn() {
            return "tenant_id";
        }

        @Override
        public boolean ignoreTable(String tableName) {
            return IGNORE_TABLES.contains(tableName);
        }
    }
}
