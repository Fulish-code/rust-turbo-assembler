;**********************************************************
; PRINCIPA.ASM
;
; Este módulo que contiene el programa principal, también
; exporta algunos de sus símbolos e importa algunos
; símbolos del otro módulo.
;
;** Este programa tiene dos módulos: ensambla MODULO.ASM y
;** PRINCIPA.ASM y enlázalos juntos desde la consola de la app:
;**
;**     TASM /zi MODULO.ASM
;**     TASM /zi PRINCIPA.ASM
;**     TLINK /v PRINCIPA MODULO
;**
;** En el manual el código de los procedimientos aparece como "...";
;** aquí esas líneas son comentarios para que ensamble.
;**********************************************************

;****** CÓDIGO DE INICIO **********************************

        ideal
        dosseg
        model   small
        stack   256

;****** DECLARACIÓN DE CONSTANTES SIMBÓLICAS **************

        extrn   cte2: abs               ; importa cte2
        public  cte4                    ; exporta cte4

cte3    equ     10                      ; constante local
cte4    =       20                      ; constante global

;****** VARIABLES DEL MÓDULO ******************************

        dataseg
;** CORREGIDO: el manual no declara codsal, que se usa al salir.
codsal  db      0

        extrn   dato2: word             ; importa dato2
        public  dato4                   ; exporta dato4

dato3   db      ?                       ; variable local
dato4   db      ?                       ; variable global

;****** CÓDIGO DEL MÓDULO *********************************

        codeseg

        extrn   proc2: proc             ; importa proc2

        public  proc4                   ; exporta proc4

inicio:
        mov     ax, @data               ; Inicializa el segmento
        mov     ds, ax                  ; de datos
        ; ...

salir:
        mov     ah, 04Ch
        mov     al, [codsal]
        int     21h

;****** PROCEDIMIENTOS ************************************

;**********************************************************
; PROC3
;
; Este procedimiento es local a este módulo.
;**********************************************************
proc    proc3
        ; ...
        ret
endp    proc3

;**********************************************************
; PROC4
;
; Este procedimiento es global.
;**********************************************************
proc    proc4
        ; ...
        ret
endp    proc4

;****** CÓDIGO DE TERMINACIÓN *****************************

        end     inicio
