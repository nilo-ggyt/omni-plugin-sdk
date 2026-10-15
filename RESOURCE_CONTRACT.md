# 协议 3 的资源收据

协议 3 保留明确的 `PluginCloseRequest { intent, scope }`。协议 2 的 `ForegroundTransition` 要求完整原生图 weak nil；其 `ApplicationTermination` 在非窗口 payload/content 图释放后，已经允许受检的 OS-only 空窗口 lease。协议 3 改变两个 scope 共同的非窗口资源成功条件：显式项目角色可提供逐类型惰性空壳收据，借用框架缓存则须证明业务内容和权限已断开。这不是新增 terminal-window 豁免，普通旧窗口仍须实际析构。

这些成功条件与协议 2 不兼容，因此 package 与 `PLUGIN_PROTOCOL_VERSION` 一起升为 3.0.0 / 3。所有消费者必须使用同一个精确 SDK Git 身份；不支持协商、旧版适配或多代协议并存。

## 所有权与证明

资源按创建、强引用和业务能力分类，不能通过运行时类名决定豁免。

- 文档、历史、映射、FD、业务缓冲区、工作线程和任务必须结束实际所有权。worker 取消并 join；队列通知只持 weak/标量，结果由可取消的会话邮箱持有，关闭时同步释放。
- 项目显式构造或配置的 controller、view、cell、menu/item 与 TextKit 角色在构造入口登记到有限账本。默认要求真实 deallocation 事件和一次完整 weak 验证；父对象 nil 不能代替子角色证明。
- 具体类型可以实现受检的惰性空壳，但必须逐字段证明业务值和容量已 take/drop、正文/选择/撤销/属性业务数据已清除、delegate/target/任务与回调权限已撤销。封闭标志或文档 weak 失效均不足以构成证明。晚到入口不得重建业务缓存或任务。
- 框架按需创建的共享 editor 和内部组合对象属于借用资源，不能通过递归遍历推断为项目独占。使用公开 API 验证编辑结束、业务文本和 undo 权限清空、delegate/旧客户端关系断开，并单独记录框架存活与重复使用增长证据。不能为观察创建 editor/context/history，也不能调用私有缓存清理。
- 观察器只持有 bounded weak/scalar 元数据，不延长被观察对象生命；区分生命周期与地址复用。旧笔记交接尚未完成的项目角色必须继续追踪，不能被下一次窗口关闭漏掉。

## 两阶段关闭

可失败阶段先完成表格/IME 输入提交、保存与正常 responder 辞任。失败保留原 owner、草稿和键盘入口，不提前清空业务值。Host 接受唯一 intent 后才进入不可逆释放；此阶段证明失败则保持 Closing、阻止后继并报告，不能伪装成可恢复预检或补发成功。

`ForegroundTransition` 完成必须通过上述资源收据，并观察旧项目窗口实际释放，之后才能创建原后继。`ApplicationTermination` 必须对应 Host 已接受的真实退出；同样释放业务资源后，可允许已关闭、不可见/非 key、无 content/delegate/responder/项目关联与业务字段的框架 window lease 留到进程实际退出。该 receipt 不是窗口 deallocation，也不能授权普通后继。

scope 只能在真实退出期间由 Host 授权单调提升，保留原 intent 与 generation，不重复输入提交、保存或 close。Host 核对 request 和交易身份、撤销 borrowed mount、在借用调用栈结束后释放 session，并只消费一次 barrier。SDK 的标量枚举不能执行或证明这些原生条件。

## 独立验收

consumer 必须覆盖失败保存、相反/重复 close、同步重入、旧回调、旧笔记交接、共享 search/Find/table 焦点、普通交接与菜单/程序退出。终止父测试独立核对保存字节与真实进程退出。资源计数归零、重复使用缓存稳定和完整产品内存/性能验收分别记录；低内存、窗口不可见、队列轮次或经过时间不能替代资源收据。

portable SDK 测试只验证协议拒绝、所有权路由、scope 和 once-only 权限；不会构造假 `NSView` 或宣称 AppKit 资源释放已通过。协议 2 的历史失败与成功结果按原始版本保留，不能被本规范重新标记为协议 3 的验收结果。
