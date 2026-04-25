# Implementation Status

## Фаза 1 — Базовая библиотека (Задание 1) [100%]

- [x] Тип `Thermometer` с конструктором и методом `temperature()`
- [x] Тип `Socket` с включением/выключением и мощностью
- [x] Тип `SmartDevice` (enum-обёртка)
- [x] Тип `Room` с массивом устройств и выводом отчёта
- [x] Тип `SmartHome` с массивом комнат и выводом отчёта
- [x] Пример-приложение с выводом отчёта и переключением розеток
- [x] Модульные тесты

## Фаза 2 — Доработка (Задание 2) [100%]

### Обработка ошибок
- [x] Заменить паники на `Option` в методах получения комнаты по ключу
- [x] Заменить паники на `Option` в методах получения устройства по ключу

### Хранение объектов
- [x] Заменить `Vec<SmartDevice>` на `HashMap<String, SmartDevice>` в `Room`
- [x] Заменить `Vec<Room>` на `HashMap<String, Room>` в `SmartHome`
- [x] `Debug` реализован через `#[derive(Debug)]` на всех типах
- [x] Динамическое добавление/удаление устройств в комнату (`add_device`/`remove_device`)
- [x] Динамическое добавление/удаление комнат в дом (`add_room`/`remove_room`)
- [x] Метод `SmartHome::get_device(room, device) -> Result<&SmartDevice, SmartHomeError>`
- [x] Тип ошибки `SmartHomeError` реализует `std::error::Error`
- [x] `From<Socket>` и `From<Thermometer>` для `SmartDevice`
- [x] Макрос `room!(name, key => device, ...)` для создания комнат

### Отчёт
- [x] Трейт `Report` с методом `report() -> String`
- [x] Реализация `Report` на `SmartDevice`, `Room`, `SmartHome`

### Тесты
- [x] Модульные тесты обновлены под новый API (27 тестов)
- [x] Интеграционные тесты обновлены (14 тестов)

### Пример-приложение
- [x] Демонстрация динамического добавления/удаления комнат
- [x] Демонстрация динамического добавления/удаления устройств
- [x] Функция `print_report<R: Report>` для вывода отчёта любого уровня
- [x] Демонстрация обработки ошибок (RoomNotFound, DeviceNotFound)

## Фаза 3 — Сетевое взаимодействие (Задание 3) [100%]

### Умная розетка (TCP)
- [x] Тип `Socket` поддерживает два режима: локальный (mock) и TCP (`Socket::new_tcp`)
- [x] Методы `turn_on/turn_off/is_on/power` возвращают `Result<_, NetworkError>`
- [x] TCP-взаимодействие: синхронное, соединение per-запрос, text-based протокол
- [x] Тип ошибки `NetworkError` реализует `std::error::Error` (через `thiserror`)

### Имитатор умной розетки (`socket_emulator`)
- [x] Читает адрес для приёма TCP-соединений из аргументов командной строки
- [x] Неблокирующий `TcpListener` (`set_nonblocking(true)`) + polling loop
- [x] Хранит состояние розетки в `Arc<Mutex<SocketState>>`
- [x] Каждый клиент обслуживается в отдельном потоке (несколько клиентов одновременно)

### Умный термометр (UDP)
- [x] Тип `Thermometer` поддерживает два режима: локальный (mock) и UDP (`Thermometer::new_udp`)
- [x] Метод `temperature()` возвращает `Result<f32, NetworkError>`
- [x] UDP-поток: фоновый поток запускается при создании, завершается через `Drop`
- [x] Фоновый поток использует таймаут на `recv_from` для проверки флага останова
- [x] `NoDataReceived` — ошибка до получения первого пакета

### Имитатор умного термометра (`thermo_emulator`)
- [x] Читает адрес назначения и период отправки из конфиг-файла (`thermo_emulator.conf`)
- [x] Неблокирующий `UdpSocket` (`set_nonblocking(true)`)
- [x] Отправляет псевдослучайное значение температуры с указанной периодичностью

### Пример demo_network
- [x] Дополнительный пример `examples/demo_network.rs`
- [x] Выводит отчёт о состоянии дома когда имитаторы запущены
- [x] Показывает ошибки (`Connection refused`, `NoDataReceived`) когда не запущены
- [x] `SmartDevice::report()` встраивает сообщение об ошибке в строку отчёта

### Тесты
- [x] `test_udp_thermometer_no_data` — проверяет `NoDataReceived`
- [x] `test_udp_thermometer_receives_data` — полный round-trip UDP test
- [x] Все тесты проходят: 27 unit + 14 integration + 3 doctests
- [x] `cargo clippy -- -D warnings` — чисто
- [x] `cargo fmt --check` — чисто
