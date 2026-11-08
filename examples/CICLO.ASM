;**********************************************************
; CICLO.ASM
;
; Este programa despliega los dígitos del 0 al 9 usando la
; instrucción repetitiva loop. El pseudocódigo es:
;
;   CX = 10
;   DL = '0'
;   do
;   {
;      despliega DL y un espacio
;      DL++
;   }
;   while(--CX > 0)
;**********************************************************

;****** CÓDIGO DE INICIO **********************************

        ideal
        dosseg
        model   small
        stack   256

;****** VARIABLES DEL PROGRAMA ****************************

        dataseg
codsal  db      0

;****** CÓDIGO DEL PROGRAMA *******************************

        codeseg
inicio:
        mov     ax, @data               ; Inicializa el
        mov     ds, ax                  ; segmento de datos

        mov     cx, 10                  ; CX = 10
        mov     dl, '0'                 ; DL = '0'
do:                                     ; do
                                        ; {
        mov     ah, 02h                 ;    Despliega DL
        int     21h
        push    dx                      ;    Despliega un espacio
        mov     dl, ' '
        int     21h
        pop     dx
        inc     dl                      ;    DL++
        loop    do                      ; }
                                        ; while(--CX > 0)

salir:
        mov     ah, 04Ch
        mov     al, [codsal]
        int     21h

;****** CÓDIGO DE TERMINACIÓN *****************************

        end     inicio
