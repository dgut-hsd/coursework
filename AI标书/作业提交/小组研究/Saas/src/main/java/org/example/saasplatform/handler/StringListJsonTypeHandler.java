package org.example.saasplatform.handler;

import com.alibaba.fastjson2.JSON;
import org.apache.ibatis.type.BaseTypeHandler;
import org.apache.ibatis.type.JdbcType;

import java.sql.CallableStatement;
import java.sql.PreparedStatement;
import java.sql.ResultSet;
import java.sql.SQLException;
import java.util.Collections;
import java.util.List;

/**
 * MyBatis TypeHandler: transparently converts {@code List<String>} ↔ JSON string.
 * <p>
 * Write: List<String> → FastJSON → JSON string → DB (JSON column)
 * Read:  DB JSON string → FastJSON → List<String>
 */
public class StringListJsonTypeHandler extends BaseTypeHandler<List<String>> {

    @Override
    public void setNonNullParameter(PreparedStatement ps, int i,
                                    List<String> parameter, JdbcType jdbcType) throws SQLException {
        ps.setString(i, JSON.toJSONString(parameter));
    }

    @Override
    public List<String> getNullableResult(ResultSet rs, String columnName) throws SQLException {
        String json = rs.getString(columnName);
        return parse(json);
    }

    @Override
    public List<String> getNullableResult(ResultSet rs, int columnIndex) throws SQLException {
        String json = rs.getString(columnIndex);
        return parse(json);
    }

    @Override
    public List<String> getNullableResult(CallableStatement cs, int columnIndex) throws SQLException {
        String json = cs.getString(columnIndex);
        return parse(json);
    }

    private List<String> parse(String json) {
        if (json == null || json.isBlank()) {
            return Collections.emptyList();
        }
        return JSON.parseArray(json, String.class);
    }
}
