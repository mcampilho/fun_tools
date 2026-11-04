# rfetch

Ferramenta de informação do sistema rápida e multiplataforma (Windows e Linux), ao estilo do neofetch.

[English](rfetch.md)

Mostra: sistema operativo e arquitetura, kernel, tempo ligado, shell, CPU, memória, disco do
sistema, utilizador Git, IP público, localização e tempo (meteorologia), fuso horário, idioma e
disposição do teclado.

## Compilar

```bash
cargo build --release --bin rfetch
```

O executável fica em `target/release/rfetch` (`rfetch.exe` no Windows).

- **Windows:** precisa das ferramentas de compilação MSVC (`link.exe`, `cl.exe`) no ambiente.
- **Linux:** precisa de um compilador de C (`gcc` ou `clang`), usado pela biblioteca de TLS.

## Utilização

```text
rfetch [opções]

  -o, --offline       Não usar a rede (sem IP, localização nem tempo)
  -r, --refresh       Ignorar os dados de rede guardados e voltar a obtê-los
      --color         Usar sempre cores
      --no-color      Nunca usar cores (também: variável de ambiente NO_COLOR)
  -l, --lang <código> Idioma: en, pt, es (também: RFETCH_LANG, RTOOLS_LANG)
      --logo <nome>   Usar outro logótipo (--logo list mostra todos)
  -h, --help          Mostrar esta ajuda
  -V, --version       Mostrar a versão
```

## Logótipos

![Todos os logótipos](rfetch-logos.png)

Os logótipos são pixel art em 256 cores: dois pixels por carácter, com meios-blocos (`▀ ▄`) e
células coloridas. Um carácter com os dois pixels da mesma cor é um espaço com fundo colorido, e
não um `█`: nos terminais em que a fonte é mais baixa do que a linha (a consola clássica, o
ConEmu, o ConsoleZ), os blocos deixam riscos escuros entre linhas. Sem cores, usam-se
`█ ▀ ▄` e os detalhes claros ficam como buracos para a forma continuar legível.

Terminais com tratamento próprio:

- O **ConsoleZ** só mostra as 16 cores básicas, por isso o logótipo usa-as.
  `RFETCH_COLORS=16` (ou `256`) força um dos modos em qualquer terminal.
- O **MobaXterm** lê a saída dos programas do Windows na página de código OEM do sistema
  (850, 437, ...) em vez de UTF-8, mesmo com a consola em UTF-8, por isso o texto é escrito
  nessa página de código.

O logótipo é escolhido automaticamente:

| Onde corre | Logótipo |
|---|---|
| Windows PowerShell 5.1 | Windows + emblema do PowerShell 5.1 |
| PowerShell 7 | Windows + emblema do PowerShell 7 |
| Git Bash, Git CMD | Windows + emblema do Git |
| MSYS2 | Windows + emblema do MSYS2 na cor de cada ambiente (MSYSTEM): MSYS roxo, UCRT64 castanho, MINGW64 azul, MINGW32 cinzento, CLANG64 laranja, CLANGARM64 verde |
| Cygwin | Windows + emblema do Cygwin |
| cmd, outros | Windows |
| Linux | Logótipo da distribuição: Ubuntu, Debian, Mint, Pop!_OS, Fedora, Red Hat, AlmaLinux, Rocky Linux, CentOS, Arch (também EndeavourOS), Manjaro, Alpine, openSUSE, Gentoo, Void, NixOS; o Tux para as restantes |
| WSL | Logótipo da distribuição + pequeno emblema do Windows |

O ambiente descobre-se pelos processos-pai: a shell mais próxima e a pasta onde está o seu
executável (as instalações do Git for Windows, do MSYS2 e do Cygwin têm ficheiros próprios que
as identificam). `--logo <nome>` força qualquer logótipo, por exemplo `--logo windows-git` ou
`--logo debian-wsl`.

Os logótipos são gerados por `tools/rfetch_logos.py` (formas simples → grelhas de texto em
`src/bin/rfetch/logo/art.rs`, mais uma imagem de pré-visualização):

```bash
python3 tools/rfetch_logos.py docs/rfetch-logos.png src/bin/rfetch/logo/art.rs
```

## Rapidez e robustez

- Os dados locais demoram algumas dezenas de milissegundos. Só se lê o que é mostrado: não se
  percorre a lista de processos, e o nome do CPU vem da instrução CPUID (a lista de CPUs do
  sysinfo demora 0,5 s ou mais no Windows).
- A rede corre em segundo plano enquanto se recolhem os dados locais, com tempos limite curtos e
  um limite total de 3,5 s. Se não responder a tempo, essas linhas não aparecem.
- O IP, a localização e o tempo ficam guardados durante 15 minutos
  (`%LOCALAPPDATA%\rfetch\net.json` no Windows, `~/.cache/rfetch/net.json` no Linux).
- A localização tenta vários fornecedores por ordem: ipwho.is, ipapi.co, ip-api.com.
  O tempo vem de open-meteo.com.
- Qualquer informação em falta é omitida em vez de parar o programa.
- Escrever para um pipe fechado (`rfetch | head`) não provoca pânico.

## Privacidade

Sem `--offline`, o IP público é enviado ao fornecedor de localização e as coordenadas
aproximadas ao open-meteo.com.

## Idiomas

Os textos do programa estão em `src/i18n.rs`, uma tabela por idioma (inglês, português europeu,
espanhol); os textos em falta ficam em inglês. O idioma vem de `--lang`, depois de
`RFETCH_LANG`, depois de `RTOOLS_LANG` (comum a todas as tools), depois do idioma do sistema. Para acrescentar um idioma, junta-o a `Lang` e a
`LANGS` e escreve a sua tabela.
