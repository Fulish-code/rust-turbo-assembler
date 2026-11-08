;**********************************************************
; MODULO.ASM
;
; Este módulo exporta algunos de sus símbolos e importa
; algunos símbolos del módulo con el programa principal.
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

;****** DECLARACIÓN DE CONSTANTES SIMBÓLICAS **************

        extrn   cte4: abs               ; importa cte4
        public  cte2                    ; exporta cte2

cte1    equ     10                      ; constante local
cte2    =       20                      ; constante global

;****** VARIABLES DEL MÓDULO ******************************

        dataseg

        extrn   dato4: byte             ; importa dato4
        public  dato2                   ; exporta dato2

dato1   db      ?                       ; variable local
dato2   dw      ?                       ; variable global

;****** CÓDIGO DEL MÓDULO *********************************

        codeseg

        extrn   proc4: proc             ; importa proc4
        public  proc2                   ; exporta proc2

;****** PROCEDIMIENTOS ************************************

;**********************************************************
; PROC1
;
; Este procedimiento es local a este módulo.
;**********************************************************
proc    proc1
        ; ...
        ret
endp    proc1

;********************************************************
; PROC2
;
; Este procedimiento es global.
;*******************************************************
proc    proc2
        ; ...
        ret
endp    proc2

;****** CÓDIGO DE TERMINACIÓN ***************************

        end
