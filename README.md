# StellarContract AMM

Contrato inteligente Soroban que implementa un Automated Market Maker (AMM) para el intercambio de tokens en la red Stellar, diseñado específicamente para el token BOB.

## Características

- Pool de liquidez descentralizado
- Intercambios automáticos con pricing dinámico (fórmula x*y=k)
- Comisiones configurables
- Protección contra slippage
- Sistema de cotizaciones en tiempo real

## Estructura del Proyecto

```
stellar-contract/
├── src/
│   ├── lib.rs          # Contrato AMM principal
│   └── test.rs         # Suite de tests completos
├── test_snapshots/     # Snapshots de tests de Soroban
├── Cargo.toml         # Configuración del paquete Rust
├── .env               # Variables de entorno (configurar después del deploy)
├── .gitignore         # Archivos ignorados por Git
├── Makefile           # Comandos de build automatizados
└── README.md          # Esta documentación

# Estructura del workspace padre:
../../
├── target/            # Archivos compilados (WASM aquí)
│   └── wasm32-unknown-unknown/release/
│       ├── stellar_contract.wasm           # WASM original
│       └── stellar_contract.optimized.wasm # WASM optimizado
├── Cargo.toml         # Configuración del workspace
└── Cargo.lock         # Lock de dependencias
```

## Arquitectura del Contrato

### Funciones Principales

```rust
// Inicialización única del contrato
initialize(admin: Address, token_a: Address, token_b: Address, fee_bps: u32)

// Gestión de liquidez
deposit(to: Address, amount_a: i128, amount_b: i128)

// Intercambios de tokens
swap(from: Address, token_in: Address, amount_in: i128, min_amount_out: i128) -> i128

// Consultas
get_reserves() -> (i128, i128)
quote_swap(token_in: Address, amount_in: i128) -> i128
get_contract_info() -> (Address, Address, Address, u32)
set_fee(admin: Address, new_fee_bps: u32)
```

### Casos de Uso

#### 1. Proveedor de Liquidez
```rust
// Depositar 1000 BOB + 500 USDC en el pool
client.deposit(user_address, 500_0000000, 1000_0000000);
```

#### 2. Intercambio de Tokens
```rust
// Intercambiar 100 BOB por USDC
let usdc_received = client.swap(
    user_address, 
    bob_token_address, 
    100_0000000,  // 100 BOB
    95_0000000    // Mínimo 95 USDC
);
```

#### 3. Consultas de Mercado
```rust
// Ver reservas actuales del pool
let (usdc_reserve, bob_reserve) = client.get_reserves();

// Cotizar precio sin ejecutar
let quoted_usdc = client.quote_swap(bob_token_address, 100_0000000);
```

## Instalación y Despliegue

### Requisitos
- **Rust**: Compilador Rust con target `wasm32-unknown-unknown`
- **Soroban CLI**: Herramientas de línea de comandos para Stellar
- **Cuenta Stellar Testnet**: Con fondos XLM para fees de transacción

### Instalación de Dependencias

```bash
# Instalar target WASM para Rust
rustup target add wasm32-unknown-unknown

# Verificar instalación de Soroban CLI
soroban --version

# Configurar red testnet
soroban network add testnet \
  --rpc-url https://soroban-testnet.stellar.org \
  --network-passphrase "Test SDF Network ; September 2015"
```

### Compilación

```bash
# Compilar para WASM (desde el directorio raíz del workspace)
cd ../../
cargo build --target wasm32-unknown-unknown --release

# Optimizar WASM
soroban contract optimize --wasm target/wasm32-unknown-unknown/release/stellar_contract.wasm

# El archivo optimizado se generará en:
# target/wasm32-unknown-unknown/release/stellar_contract.optimized.wasm
```

### Configuración de Identity

```bash
# Generar nueva identity para admin
soroban keys generate admin --network testnet

# Ver la dirección pública generada
soroban keys address admin

# Fondear cuenta con XLM de testnet
# Visita: https://friendbot.stellar.org/
# Envía XLM a tu dirección pública
```

### Despliegue en Testnet

```bash
# Desplegar el contrato
soroban contract deploy \
  --wasm ../../target/wasm32-unknown-unknown/release/stellar_contract.optimized.wasm \
  --source admin \
  --network testnet

# Guardar el CONTRACT_ID que se muestra en la salida
```

### Inicialización del Contrato

```bash
# Inicializar AMM (reemplazar valores con los reales)
soroban contract invoke \
  --id <CONTRACT_ID> \
  --source admin \
  --network testnet \
  -- initialize \
  --admin <ADMIN_PUBLIC_KEY> \
  --token_a_address <USDC_CONTRACT_ADDRESS> \
  --token_b_address <BOB_CONTRACT_ADDRESS> \
  --fee_bps 30
```

## Uso del Contrato

### Configurar Variables de Entorno
Después del despliegue exitoso, actualiza tu archivo `.env`:

```bash
# Ejemplo de .env después del despliegue
AMM_CONTRACT_ID=CABC123...XYZ789  # ID obtenido del deploy
ADMIN_PUBLIC_KEY=GABC123...XYZ789  # Dirección obtenida con 'soroban keys address admin'
BOB_TOKEN_ADDRESS=CBOB123...XYZ789  # Dirección del token BOB
USDC_TOKEN_ADDRESS=CUSDC123...XYZ789  # Dirección del token USDC
```

### Comandos de Interacción

#### Agregar Liquidez Inicial
```bash
soroban contract invoke \
  --id $AMM_CONTRACT_ID \
  --source admin \
  --network testnet \
  -- deposit \
  --to $ADMIN_PUBLIC_KEY \
  --amount_a 500000000 \    # 50 USDC (7 decimales)
  --amount_b 1000000000     # 100 BOB (7 decimales)
```

#### Intercambiar Tokens

```bash
# 1. Obtener cotización (solo lectura)
soroban contract invoke \
  --id $AMM_CONTRACT_ID \
  --network testnet \
  -- quote_swap \
  --token_in_address $BOB_TOKEN_ADDRESS \
  --amount_in 100000000   # 10 BOB

# 2. Ejecutar intercambio real
soroban contract invoke \
  --id $AMM_CONTRACT_ID \
  --source admin \
  --network testnet \
  -- swap \
  --from $ADMIN_PUBLIC_KEY \
  --token_in_address $BOB_TOKEN_ADDRESS \
  --amount_in 100000000 \     # 10 BOB
  --min_amount_out 45000000   # Mínimo 4.5 USDC esperados
```

#### Consultar Estado del Pool

```bash
# Ver reservas actuales
soroban contract invoke \
  --id $AMM_CONTRACT_ID \
  --network testnet \
  -- get_reserves

# Ver información del contrato
soroban contract invoke \
  --id $AMM_CONTRACT_ID \
  --network testnet \
  -- get_contract_info

# Cambiar comisiones (solo admin)
soroban contract invoke \
  --id $AMM_CONTRACT_ID \
  --source admin \
  --network testnet \
  -- set_fee \
  --admin $ADMIN_PUBLIC_KEY \
  --new_fee_bps 25    # Cambiar a 0.25%
```

## Integración con Token BOB

### Requisitos del Token
El contrato AMM puede trabajar con cualquier token que implemente el estándar de token Soroban:
- Token BOB debe ser un contrato Soroban válido
- Debe implementar las funciones estándar: `transfer`, `balance`, etc.
- Se requiere liquidez inicial para establecer el precio base

### Interacción con el Ecosystem Stellar
- **Compatibilidad**: Funciona con activos nativos de Stellar envueltos
- **Escalabilidad**: Diseñado para manejar grandes volúmenes de transacciones
- **Seguridad**: Utiliza las características de seguridad nativas de Soroban

## Arquitectura del Sistema

```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   Token BOB     │    │   AMM Contract   │    │   Token USDC    │
│                 │    │                  │    │                 │
│ • Balance       │◄──►│ • Pool Reserves  │◄──►│ • Balance       │
│ • Transfer      │    │ • Swap Logic     │    │ • Transfer      │
│ • Approve       │    │ • Fee Management │    │ • Approve       │
└─────────────────┘    │ • Price Calc     │    └─────────────────┘
                       └──────────────────┘
                              ▲
                              │
                    ┌─────────────────┐
                    │   Frontend UI   │
                    │                 │
                    │ • React Hooks   │
                    │ • Price Display │
                    │ • Swap Interface│
                    │ • LP Management │
                    └─────────────────┘
```

## Seguridad

### Características de Seguridad
- Autorización requerida para todas las transacciones
- Protección contra slippage excesivo
- Administración controlada de comisiones
- Validación de entrada y prevención de overflow
- Protección contra ataques de reentrada

### Características Técnicas
- **Gas Optimizado**: Uso eficiente de almacenamiento Soroban
- **Modular Design**: Fácil extensión para nuevos tokens
- **Batch Operations**: Múltiples operaciones en una transacción

## Tests

### Ejecutar Tests Locales
```bash
# Ejecutar todos los tests
cargo test

# Ejecutar tests con output detallado
cargo test -- --nocapture

# Ejecutar un test específico
cargo test test_initialize

# Ver coverage de tests
cargo test --verbose
```

### Tests Incluidos
El contrato incluye una suite completa de tests que cubren:

- ✅ **test_initialize**: Inicialización correcta del contrato
- ✅ **test_double_initialization**: Prevención de doble inicialización
- ✅ **test_deposit_and_get_reserves**: Depósito de liquidez y consulta de reservas
- ✅ **test_swap**: Intercambio de tokens BOB ↔ USDC
- ✅ **test_quote_swap**: Cotizaciones sin ejecutar transacciones
- ✅ **test_set_fee**: Cambio de comisiones por el administrador

### Resultados Esperados
```bash
# Todos los tests deben pasar
running 6 tests
test test::test_initialize ... ok
test test::test_double_initialization ... ok
test test::test_deposit_and_get_reserves ... ok
test test::test_swap ... ok
test test::test_quote_swap ... ok
test test::test_set_fee ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

## Estado de Desarrollo

| Componente | Estado | Descripción |
|------------|--------|-------------|
| 🔐 Contrato Base | ✅ Completo | Lógica AMM implementada y testada |
| 🧪 Tests Unitarios | ✅ Completo | 6 tests, 100% passing |
| 📚 Documentación | ✅ Completo | README detallado con ejemplos |
| 🔧 Scripts Deploy | ✅ Completo | Comandos Soroban documentados |

## Contribuciones

Este proyecto está diseñado para:
- **Hackathons**: Base sólida para proyectos DeFi
- **Educación**: Ejemplo completo de contrato Soroban
- **Producción**: Con auditoría de seguridad recomendada

## Recursos

- [Documentación de Soroban](https://developers.stellar.org/docs/build/smart-contracts)
- [Ejemplos de Soroban](https://github.com/stellar/soroban-examples)
- [Token Wrapper para Stellar Assets](https://developers.stellar.org/docs/build/smart-contracts/example-contracts/tokens)

---

**Proyecto StellarContract AMM - Un AMM completo para el ecosistema Stellar**

## Variables de Entorno

Después del despliegue, actualiza tu archivo `.env` con los valores reales:

```bash
# Configuración de red
STELLAR_NETWORK=testnet
SOROBAN_RPC_URL=https://soroban-testnet.stellar.org

# Direcciones del contrato (actualizar después del despliegue)
AMM_CONTRACT_ID=tu_contract_id_generado
BOB_TOKEN_ADDRESS=direccion_del_token_bob
USDC_TOKEN_ADDRESS=direccion_del_token_usdc

# Configuración de deployment
ADMIN_SECRET_KEY=tu_clave_privada_admin
ADMIN_PUBLIC_KEY=tu_direccion_publica_admin

# Configuración del pool
INITIAL_FEE_BPS=30

# Opcional: Cuenta de funding para testing
FUNDING_SECRET_KEY=
FUNDING_PUBLIC_KEY=

# Environment
NODE_ENV=development
```

### Variables Principales
- `AMM_CONTRACT_ID`: ID del contrato AMM desplegado (se obtiene después del deploy)
- `ADMIN_PUBLIC_KEY`: Dirección pública del administrador (se obtiene con `soroban keys address admin`)
- `BOB_TOKEN_ADDRESS`: Dirección del contrato del token BOB
- `USDC_TOKEN_ADDRESS`: Dirección del contrato USDC en testnet
- `INITIAL_FEE_BPS`: Comisión inicial del pool (30 = 0.3%)

## Troubleshooting

### Problemas Comunes

#### Error de compilación: "target not found"
```bash
# Solución: Instalar target WASM
rustup target add wasm32-unknown-unknown
```

#### Error: "workspace dependency not found"
```bash
# Solución: Verificar que las dependencias estén correctamente especificadas
# En Cargo.toml debe ser:
soroban-sdk = "22.0.8"
# NO: soroban-sdk = { workspace = true }
```

#### Error de despliegue: "insufficient funds"
```bash
# Solución: Fondear cuenta con XLM
# 1. Obtener dirección pública: soroban keys address admin
# 2. Visitar: https://friendbot.stellar.org/
# 3. Enviar XLM a tu dirección
```

#### Error: "Contract already initialized"
```bash
# El contrato ya fue inicializado anteriormente
# No se puede llamar initialize() más de una vez
```

### Verificación del Despliegue

```bash
# Verificar que el contrato fue desplegado correctamente
soroban contract invoke \
  --id $AMM_CONTRACT_ID \
  --network testnet \
  -- get_contract_info

# Debería retornar: (admin_address, token_a_address, token_b_address, fee_bps)
```

### Logs y Debug

```bash
# Para ver logs detallados durante las operaciones
export RUST_LOG=debug

# Para transacciones fallidas, revisar el hash en:
# https://stellar.expert/explorer/testnet
```

## Comandos de Desarrollo

### Testing Local
```bash
# Ejecutar tests
cargo test

# Ejecutar tests con output detallado
cargo test -- --nocapture

# Ejecutar un test específico
cargo test test_initialize
```

### Recompilación Rápida
```bash
# Script para recompilar y optimizar rápidamente
cargo build --target wasm32-unknown-unknown --release && \
soroban contract optimize --wasm ../../target/wasm32-unknown-unknown/release/stellar_contract.wasm
```

### Limpieza
```bash
# Limpiar archivos de compilación
cargo clean

# Limpiar solo target local (si existe)
rm -rf target/
```

## Información del Proyecto

### Versiones y Compatibilidad
- **Rust Edition**: 2021
- **Soroban SDK**: 22.0.8
- **Target**: wasm32-unknown-unknown
- **Red Soportada**: Stellar Testnet (configuración para Mainnet disponible)

### Características Técnicas
- **Tamaño WASM Original**: ~16KB
- **Tamaño WASM Optimizado**: ~12KB
- **Funciones Públicas**: 7 (initialize, deposit, swap, get_reserves, quote_swap, get_contract_info, set_fee)
- **Almacenamiento**: Persistente con claves tipadas
- **Decimales Soportados**: 7 (estándar Stellar)

### Consideraciones de Seguridad
- **Autorización**: Requerida para todas las operaciones de modificación
- **Validación**: Entrada validada y protección contra overflow
- **Slippage**: Protección configurable por el usuario
- **Admin Controls**: Solo el administrador puede cambiar parámetros críticos
- **Re-entrancy**: Protegido por el diseño de Soroban

### Limitaciones Conocidas
- **LP Tokens**: No implementados (simplificación para el ejemplo)
- **Multi-Pool**: Diseñado para un solo par de tokens
- **Fee Distribution**: Las comisiones se mantienen en el pool
- **Price Oracle**: No integrado (usa solo reservas del pool)

### Roadmap de Mejoras
- [ ] Implementación de LP tokens
- [ ] Soporte para múltiples pools
- [ ] Integración con price oracle
- [ ] Distribución de fees a LP providers
- [ ] Interfaz web completa
- [ ] Auditoría de seguridad

### Enlaces Útiles
- **Stellar Expert (Testnet)**: https://stellar.expert/explorer/testnet
- **Soroban Documentation**: https://developers.stellar.org/docs/build/smart-contracts
- **Stellar Friendbot**: https://friendbot.stellar.org/
- **Soroban Examples**: https://github.com/stellar/soroban-examples

---

**⚠️ Aviso Legal**: Este contrato es un ejemplo educativo. Para uso en producción se recomienda una auditoría completa de seguridad.

**📧 Soporte**: Para reportar issues o contribuir, usa las herramientas de desarrollo estándar de Rust/Cargo.

**🏆 Proyecto StellarContract AMM - Implementación completa de AMM para el ecosistema Stellar**
