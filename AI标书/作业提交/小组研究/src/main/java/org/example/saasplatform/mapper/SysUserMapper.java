package org.example.saasplatform.mapper;

import com.baomidou.mybatisplus.core.mapper.BaseMapper;
import org.apache.ibatis.annotations.Mapper;
import org.example.saasplatform.entity.SysUser;

@Mapper
public interface SysUserMapper extends BaseMapper<SysUser> {
    // Inherits 17 built-in CRUD methods from BaseMapper
}
