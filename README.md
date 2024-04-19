# RCLI






### 项目依赖
- ```cargo add clap --features derive```
    - 开启derive允许使用 #[derive(Clap)] 宏来自动导出命令行参数解析器。可以更轻松地定义和管理命令行参数，从结构体中自动生成相应的解析器。
    - derive feature依赖`syn`及`quote`两个库，因此不是默认开启的特性
- ```cargo add serde --features derive```
- ```cargo add serde-json --features derive```
- ```cargo add csv```



### 项目初始化

```bash
pre-commit install
```

deny初始化？
```bash
cargo deny init
```


### 环境要求

开发环境：
- 可翻墙（pre-commit）
