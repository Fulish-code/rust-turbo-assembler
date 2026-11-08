;**********************************************************
; MAYOR3_2.ASM
;
; Este programa encuentra el mayor de tres datos signados
; almacenados en variables de una palabra. El programa
; utiliza una macro para encontrar al mayor de los tres
; datos
;**********************************************************

;****** CÓDIGO DE INICIO **********************************

        ideal
        dosseg
        model   small
        stack   256

;****** MACROS ********************************************

;**********************************************************
; MAYOR3
;
; Esta macro encuentra el mayor de tres datos signados
; dados por los parámetros dato1, dato2 y dato3. El mayor
; de los tres datos queda en el registro AX.
;**********************************************************
macro   mayor3 dato1, dato2, dato3
        local   sigcmp, fincmp

        mov     ax, [dato1]             ; AX = dato1
        cmp     ax, [dato2]             ; if(AX > dato2)
        jg      sigcmp                  ;    goto sigcmp
        mov     ax, [dato2]             ; else AX = dato2
sigcmp:
        cmp     ax, [dato3]             ; if(AX > dato3)
        jg      fincmp                  ;    goto fincmp
        mov     ax, [dato3]             ; else AX = dato3
fincmp:
endm    mayor3

;****** VARIABLES DEL PROGRAMA ****************************

        dataseg
codsal  db      0
dato1   dw      ?
dato2   dw      ?
dato3   dw      ?
mayor   dw      ?

;****** CÓDIGO DEL PROGRAMA *******************************

        codeseg
inicio:
        mov     ax, @data               ; Inicializa el
        mov     ds, ax                  ; segmento de datos

        mayor3  dato1, dato2, dato3

        mov     [mayor], ax             ; mayor = AX

salir:

        mov     ah, 04Ch
        mov     al, [codsal]
        int     21h

;****** CÓDIGO DE TERMINACIÓN *****************************

        end     inicio
