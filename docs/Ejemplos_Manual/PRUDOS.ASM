;**********************************************************
; PRUDOS.ASM
;
; Este programa prueba las macros del archivo DOS_MAC.INC
; (capítulo 8), que usan las rutinas del DOS (int 21h):
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

        include "dos_mac.inc"

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
@@whi:  mov     dl, [si]                ; while([SI] != 0)
        cmp     dl, 0                   ; {
        je      @@endwhi
        aputchar                        ;    despliega [SI]
        inc     si                      ;    SI++
        jmp     @@whi                   ; }
@@endwhi:
        agetchar                        ; Lee un carácter y lo despliega

salir:
        mov     ah, 04Ch
        mov     al, [codsal]
        int     21h

;****** CÓDIGO DE TERMINACIÓN *****************************

        end     inicio
