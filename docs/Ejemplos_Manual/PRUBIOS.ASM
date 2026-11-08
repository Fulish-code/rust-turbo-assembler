;**********************************************************
; PRUBIOS.ASM
;
; Este programa prueba las macros del archivo BIOS_MAC.INC
; (capítulo 8), que usan las rutinas del BIOS (int 10h e int 16h):
; despliega un mensaje carácter por carácter, lee una tecla
; y la despliega.
;
;** El programa principal no viene en el manual: se agregó para
;** poder ensamblar, ejecutar y depurar (TD) las macros.
;**********************************************************

;****** CÓDIGO DE INICIO **********************************

        ideal
        dosseg
        model   small
        stack   256

;****** INCLUSIÓN DE ARCHIVOS *****************************

        include "bios_mac.inc"

;****** VARIABLES DEL PROGRAMA ****************************

        dataseg
codsal  db      0
msj     db      'Oprime una tecla: ', 0

;****** CÓDIGO DEL PROGRAMA *******************************

        codeseg
inicio:
        mov     ax, @data               ; Inicializa el
        mov     ds, ax                  ; segmento de datos

        mov     si, offset msj          ; SI = msj
@@whi:  mov     al, [si]                ; while([SI] != 0)
        cmp     al, 0                   ; {
        je      @@endwhi
        aputch                          ;    despliega [SI]
        inc     si                      ;    SI++
        jmp     @@whi                   ; }
@@endwhi:
        agetch                          ; Lee un carácter (AL)
        aputch                          ; y lo despliega

salir:
        mov     ah, 04Ch
        mov     al, [codsal]
        int     21h

;****** CÓDIGO DE TERMINACIÓN *****************************

        end     inicio
