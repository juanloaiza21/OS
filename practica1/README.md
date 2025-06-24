## Dataset de Viajes en Taxi

Descripción del Dataset

Este dataset contiene registros de viajes en taxi en la ciudad de Nueva York durante el mes de junio de 2020. Los datos incluyen información detallada sobre cada viaje, como horarios, ubicaciones, distancias, tarifas y métodos de pago.

Estructura del Dataset

El archivo `trips.csv` contiene los siguientes campos:

1. VendorID: Identificador del proveedor de servicios de taxi (1 o 2)
   - Valores válidos: 1, 2

2. tpep_pickup_datetime: Fecha y hora de recogida del pasajero
   - Formato: YYYY-MM-DD HH:MM:SS

3. tpep_dropoff_datetime: Fecha y hora de llegada del pasajero
   - Formato: YYYY-MM-DD HH:MM:SS

4. passenger_count: Número de pasajeros en el viaje
   - Rango: 0-6 (0 indica un viaje sin pasajeros registrados)

5. trip_distance: Distancia del viaje en millas
   - Rango: 0-30.69 (0 indica viajes sin movimiento)

6. RatecodeID: Código de tarifa aplicada
   - 1 = Tarifa estándar
   - 2 = JFK
   - 3 = Newark
   - 4 = Negociada
   - 5 = Grupo
   - 6 = Tarifa por hora

7. store_and_fwd_flag: Indica si el viaje fue guardado en la memoria del vehículo antes de ser enviado al proveedor
   - Valores: Y (Sí), N (No)

8. PULocationID: ID de la zona de recogida (Taxi Zone)
   - Rango: 1-265

9. DOLocationID: ID de la zona de destino (Taxi Zone)
   - Rango: 1-265

10. payment_type: Método de pago
    - 1 = Tarjeta de crédito
    - 2 = Efectivo
    - 3 = Sin cargo
    - 4 = Disputa
    - 5 = Desconocido
    - 6 = Viaje anulado

11. fare_amount: Tarifa base del viaje en USD
    - Rango: -13.5 a 122.23 (valores negativos indican reembolsos)

12. extra: Cargos adicionales en USD
    - Rango: -0.5 a 3

13. mta_tax: Impuesto MTA en USD
    - Rango: -0.5 a 0.5

14. tip_amount: Propina en USD
    - Rango: 0-300

15. tolls_amount: Peajes en USD
    - Rango: 0-23.5

16. improvement_surcharge: Cargo por mejora en USD
    - Rango: -0.3 a 0.3

17. total_amount: Monto total pagado en USD
    - Rango: -15.3 a 131.6

18. congestion_surcharge: Cargo por congestión en USD
    - Rango: -2.5 a 2.5

19. Index**: Número de índice del registro
    - Rango: 1-650

## Aplicación de Consulta de Viajes

Descripción
Aplicación gráfica en Rust para consultar y filtrar registros de viajes desde un archivo CSV. Permite búsquedas combinadas por índice, fecha y costo.

Criterios de Búsqueda Implementados

1. Búsqueda por Índice
- Campo: index (ID único generado secuencialmente)
- Tipo: Búsqueda exacta
- Formato: Número entero positivo
- Ejemplo: 128 → Solo el viaje con índice 128

2. Búsqueda por Fecha (Opcional)
- Campo: pickup (fecha de recogida)
- Componentes:
  - Día: 1-31 (ajuste automático por mes/año)
  - Mes: 1-12
  - Año: 2009-2025
- Activación: Checkbox "Usar fecha"
- Precisión: Comparación exacta (YYYY-MM-DD)

3. Búsqueda por Costo
- Campo: total_amount
- Operadores:
  - <= (Menor o igual)
  - >= (Mayor o igual)
  - = (Igualdad exacta con tolerancia ε)
- Formato: Número decimal (ej: 15.50)

Lógica de Filtrado Combinado
Los criterios se aplican en AND lógico:
(Índice) AND (Fecha si activa) AND (Costo)

Adaptaciones Realizadas

1. Procesamiento del CSV
- Ignora cabecera: Salta la primera línea automáticamente
- Resiliencia:
  - Omite líneas mal formadas (parts.len() < 18)
  - Valores inválidos → 0 (pasajeros, distancia, monto)
- Generación de índice: Crea IDs secuenciales desde 1

2. Manejo de Fechas
- Validación inteligente:
  - Ajusta días según mes (incluye años bisiestos)
  - Selectores jerárquicos (año → mes → día)
- Formateo: Extrae solo la fecha de pickup (ignora hora)

3. Interfaz Gráfica
- Diseño responsive: 4 columnas que se adaptan
- Feedback visual:
  - Mensajes contextuales (sin búsqueda/sin resultados)
  - Tabla con bandas de color alternado
  - Formato numérico consistente (2 decimales)
- Controles especializados:
  - Combobox para selección de operadores/fechas
  - Placeholders descriptivos

4. Optimizaciones
- Carga única: Datos en memoria para filtrados rápidos
- Rendimiento:
  - Filtrado por iteradores (lazy evaluation)
  - Clonación selectiva solo de resultados
